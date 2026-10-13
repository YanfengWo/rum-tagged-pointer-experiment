"""Prepare isolated native cache experiments; never edit the default kernel."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
REVISION="77029938f999b14dbe5b0755bebdf98e51517ae6"
STAGES=["compact-expr","recursive-expr","symbol-sharing","expression-census","layout-design","typed-layout","native-dispatch","native-sort"]
def prepare(output,metadata,mode):
    study=Path(__file__).resolve().parent;repo=study.parents[2];research=study.parent
    output=output.resolve()
    assert output!=research and research not in output.parents
    assert not output.exists() or (output.is_dir() and not any(output.iterdir()))
    assert not metadata.exists()
    data=subprocess.run(["git","archive","--format=tar",REVISION],cwd=repo,check=True,capture_output=True,timeout=180).stdout
    output.mkdir(parents=True,exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(data),mode="r:") as tar:
        for m in tar:
            p=(output/m.name).resolve();assert (p==output or output in p.parents) and (m.isfile() or m.isdir())
        tar.extractall(output)
    changes={}
    def patch(rel,result):
        p=output/rel;base=p.read_bytes();new=result.encode("utf-8")
        changes[rel]={"base_sha256":hashlib.sha256(base).hexdigest(),"patched_sha256":hashlib.sha256(new).hexdigest()}
        p.write_bytes(new)
    if mode!="base":
        for stage in STAGES:shutil.copytree(research/stage,output/"research/tagged-pointer"/stage,ignore=shutil.ignore_patterns("target","__pycache__"))
        p=output/"crates/rum-core/src/expr.rs"
        extension=(research/"typed-layout/symbol-access.rs").read_text(encoding="utf-8")
        if mode.startswith("sort-"):extension+="\n"+(research/"native-sort/text-access.rs").read_text(encoding="utf-8")
        assert "unsafe" not in extension
        core=p.read_text(encoding="utf-8")
        for signature in ["    pub fn interned_id(&self) -> u32 {","    pub fn identity(&self) -> Option<u64> {"]:
            assert core.count(signature)==1
            core=core.replace(signature,"    #[inline]\n"+signature)
        patch("crates/rum-core/src/expr.rs",core+"\n"+extension)
        if mode not in {"api","sort-api"}:
            if mode.startswith("sort-"):patch_sort(output,patch,mode)
            else:patch_runtime(output,patch,mode)
    rust_files=[str(output/rel) for rel in changes if rel.endswith(".rs")]
    if rust_files:
        subprocess.run(["rustup","run","1.85.0","rustfmt","--edition","2024",*rust_files],check=True,timeout=180)
        for rel,recorded in changes.items():recorded["patched_sha256"]=hashlib.sha256((output/rel).read_bytes()).hexdigest()
    record={"upstream_commit":REVISION,"mode":mode,"overlay_path":str(output),"patched_files":changes,
            "default_checkout_unchanged":True,"integration_scope":"Native builtin cache or wide symbol Sort temporary values; evaluator Expr representation remains original"}
    metadata.parent.mkdir(parents=True,exist_ok=True);metadata.write_text(json.dumps(record,indent=2)+"\n",encoding="utf-8")
    print("PASS: isolated",mode,"source prepared")
def patch_runtime(output,patch,mode):
    root=output/"Cargo.toml";value=root.read_text(encoding="utf-8")
    excluded=["research/tagged-pointer/"+s for s in STAGES]
    value=value.replace('resolver = "3"','resolver = "3"\nexclude = '+json.dumps(excluded))
    patch("Cargo.toml",value)
    features=["raw24","safe24","packed16","tagged16"]
    for package,dep in [("rum-cli","rum-host"),("rum-host","rum-runtime")]:
        rel=f"crates/{package}/Cargo.toml";value=(output/rel).read_text(encoding="utf-8")
        lines="\n".join(f'research-dispatch-{f} = ["{dep}/research-dispatch-{f}"]' for f in features)
        value=value.replace("[dependencies]",lines+"\n\n[dependencies]")
        patch(rel,value)
    rel="crates/rum-runtime/Cargo.toml";value=(output/rel).read_text(encoding="utf-8")
    feature_lines=['[features]','research-dispatch = ["dep:rum-native-dispatch"]']
    for f in features:feature_lines.append(f'research-dispatch-{f} = ["research-dispatch"'+(f', "rum-native-dispatch/{f}"' if f!="raw24" else "")+']')
    value+="\n"+"\n".join(feature_lines)+'\n\n[dependencies.rum-native-dispatch]\npath = "../../research/tagged-pointer/native-dispatch"\noptional = true\n'
    patch(rel,value)
    rel="crates/rum-runtime/src/runtime/model/cache.rs";value=(output/rel).read_text(encoding="utf-8")
    old='pub(in crate::runtime) type BuiltinDispatchSpecCache =\n    HashMap<Symbol, Option<BuiltinSpec>, BuildHasherDefault<RuntimeCacheHasher>>;'
    assert value.count(old)==1
    new='#[cfg(not(feature = "research-dispatch"))]\n'+old+'\n\n#[cfg(feature = "research-dispatch")]\npub(in crate::runtime) type BuiltinDispatchSpecCache =\n    rum_native_dispatch::Map<Option<BuiltinSpec>, BuildHasherDefault<RuntimeCacheHasher>>;'
    patch(rel,value.replace(old,new))
def patch_sort(output,patch,mode):
    root=output/"Cargo.toml";value=root.read_text(encoding="utf-8")
    value=value.replace('resolver = "3"','resolver = "3"\nexclude = '+json.dumps(["research/tagged-pointer/"+s for s in STAGES]))
    patch("Cargo.toml",value)
    for package,dep in [("rum-cli","rum-host"),("rum-host","rum-stdlib")]:
        rel=f"crates/{package}/Cargo.toml";value=(output/rel).read_text(encoding="utf-8")
        features="\n".join(f'research-sort-{f} = ["{dep}/research-sort-{f}"]' for f in ["24","safe16","tagged16"])
        patch(rel,value.replace("[dependencies]",features+"\n\n[dependencies]"))
    rel="crates/rum-stdlib/Cargo.toml";value=(output/rel).read_text(encoding="utf-8")
    features=['research-sort = ["dep:rum-native-sort"]','research-sort-24 = ["research-sort"]',
              'research-sort-safe16 = ["research-sort", "rum-native-sort/safe16"]',
              'research-sort-tagged16 = ["research-sort", "rum-native-sort/tagged16"]']
    value=value.replace("[dependencies]","\n".join(features)+"\n\n[dependencies]")
    value+='\n[dependencies.rum-native-sort]\npath = "../../research/tagged-pointer/native-sort"\noptional = true\n'
    patch(rel,value)
    rel="crates/rum-stdlib/src/structure_ops/ordering_sets.rs";value=(output/rel).read_text(encoding="utf-8")
    old='    let mut sorted = expr_args.to_vec();\n    sorted.sort_by(compare_structural_order);\n    Ok(Expr::call(expr_head.as_ref().clone(), sorted))'
    assert value.count(old)==1
    new='    #[cfg(feature = "research-sort")]\n    if let Some(sorted) = rum_native_sort::try_sort_symbols(expr_args.as_ref()) {\n        return Ok(Expr::call(expr_head.as_ref().clone(), sorted));\n    }\n\n'+old
    patch(rel,value.replace(old,new))
if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--output",type=Path,required=True);p.add_argument("--metadata",type=Path,required=True)
    p.add_argument("--mode",choices=["base","api","raw24","safe24","packed16","tagged16","sort-api","sort-24","sort-safe16","sort-tagged16"],required=True)
    a=p.parse_args();prepare(a.output,a.metadata,a.mode)
