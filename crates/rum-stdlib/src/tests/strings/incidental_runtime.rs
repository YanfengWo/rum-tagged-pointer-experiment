use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rum_runtime::{SessionRequestError, SessionRequestFailure};

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rum-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn rum_path(path: &Path) -> String {
    let normalized = fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });
    normalized.display().to_string().replace('\\', "/")
}

fn expect_input_prompt(error: SessionRequestFailure, prompt: &str) {
    match &error.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, prompt);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}

#[test]
fn file_template_apply_catalog_capability_stays_file_write() {
    let capability = crate::catalog::STRING_BUILTINS
        .iter()
        .find(|spec| spec.name == "FileTemplateApply")
        .expect("FileTemplateApply should be cataloged")
        .capability;

    assert_eq!(capability, rum_runtime::kernel_api::Capability::FileWrite);
}

// Moved from crates/rum-runtime/src/runtime/tests/associations_aggregate_surface.rs.

#[test]
fn merge_catenate_counts_groupby_and_positionindex_cover_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Merge[{Association[a -> 1, b -> 2], Association[a -> 3]}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 4], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{Association[a -> 1], {a -> 2, b -> 3}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{a -> 1, {a -> 2, b -> 3}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{{{a -> 1}, {a -> 2, b -> 3}}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{{Association[a -> 1], {a -> 2}}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[Total][{Association[a -> 1, b -> 2], Association[a -> 3]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 4], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {Merge[{Association[a :> (x = x + 1)], Association[a :> (x = x + 1)]}, Function[v, v]], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, List[1, 2]]], 2]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {Merge[{Association[a :> (x = x + 1)], Association[a -> 2]}, HoldComplete], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, HoldComplete[List[Set[x, Plus[x, 1]], 2]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[{Association[a -> 1], Association[b -> 2]}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> {1, 2}, b -> {3}]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> Missing[], b -> {2}]]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> Missing[\"x\"], b -> Missing[]]]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    let outcome = session
        .eval_text("Catenate[Association[a -> {1}, b -> 2]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Catenate[Association[Rule[a, List[1]], Rule[b, 2]]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "invrp"
            && message.text == "The argument 2 is not a valid Association or a list."
    }));
    let outcome = session
        .eval_text("Catenate[Association[a -> Missing[], b -> 2]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Catenate[Association[Rule[a, Missing[]], Rule[b, 2]]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "invrp"
            && message.text == "The argument 2 is not a valid Association or a list."
    }));
    assert_eq!(
        session
            .eval_text("Catenate[{Association[a -> {1, 2}], {3, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Catenate[Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], 1]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Catenate[Association[a :> {x = x + 1}, b :> {x = x + 1}]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 2]"
    );
    let outcome = session
        .eval_text("x = 0; {Catenate[Association[a :> {x = x + 1}, b :> (x = x + 1)]], x}")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[Catenate[Association[RuleDelayed[a, List[Set[x, Plus[x, 1]]]], RuleDelayed[b, Set[x, Plus[x, 1]]]]], 2]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "invrp"
            && message.text == "The argument 2 is not a valid Association or a list."
    }));
    assert_eq!(
        session
            .eval_text(
                "x = 0; {Catenate[{Association[a :> (x = x + 1)], Association[b :> (x = x + 1)]}], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 2]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], Rule[b, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[Association[a -> countedX, b -> countedX, c -> countedY]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, 2], Rule[countedY, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}, {a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], Rule[b, 1], Rule[c, 0]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Counts[Association[a -> countedX, b -> countedX, c -> countedY], {countedX, countedY, countedZ}]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, 2], Rule[countedY, 1], Rule[countedZ, 0]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Counts[Association[a :> (x = x + 1), b :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, 1], Rule[2, 1]], 2]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Counts[Association[a :> (x = x + 1), b -> 1], {1, 2}], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, 2], Rule[2, 0]], 1]"
    );
    assert_eq!(
        session
            .eval_text("CountsBy[{a, bb, ccc}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, 1], Rule[2, 1], Rule[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("CountsBy[StringLength[ToString[#]] &][{a, bb, ccc}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, 1], Rule[2, 1], Rule[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, List[a]], Rule[2, List[bb, dd]], Rule[3, List[ccc]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, a], Rule[2, dd], Rule[3, ccc]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[StringLength[ToString[#]] &][{a, bb, ccc, dd}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, List[a]], Rule[2, List[bb, dd]], Rule[3, List[ccc]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, Association[Rule[first, countedX], Rule[second, countedX]]], Rule[countedY, Association[Rule[third, countedY]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {GroupBy[Association[first :> (x = x + 1), second :> (x = x + 1)], Identity], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, Association[RuleDelayed[first, Set[x, Plus[x, 1]]]]], Rule[2, Association[RuleDelayed[second, Set[x, Plus[x, 1]]]]]], 2]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Identity][Association[first -> countedX, second -> countedX, third -> countedY]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, Association[Rule[first, countedX], Rule[second, countedX]]], Rule[countedY, Association[Rule[third, countedY]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity, Identity]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, Association[Rule[first, countedX], Rule[second, countedX]]], Rule[countedY, Association[Rule[third, countedY]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity, HoldComplete]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, HoldComplete[Association[Rule[first, countedX], Rule[second, countedX]]]], Rule[countedY, HoldComplete[Association[Rule[third, countedY]]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity, Last]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, countedX], Rule[countedY, countedY]]"
    );
    assert_eq!(
        session
            .eval_text("PositionIndex[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, List[1, 3]], Rule[b, List[2]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "PositionIndex[Association[first -> countedX, second -> countedX, third -> countedY]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, List[first, second]], Rule[countedY, List[third]]]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/associations_key_ops.rs.

#[test]
fn key_predicates_take_drop_map_and_sort_cover_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyExistsQ[Association[a -> 1, b -> 2], b]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[Association[a -> 1], b]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[a -> 1, a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[Association[a -> 1, b -> 2, c -> 3], {b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2], Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[Association[a -> 1, b -> 2, c -> 3], b]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {KeyTake[Association[a :> (x = x + 1)], a], KeyDrop[Association[a :> (x = x + 1), b -> 2], b], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[Association[a -> 1, Key[a] -> 7], Key[a]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[Association[a -> 1, Key[a] -> 7, b -> 2], {Key[a], b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[Key[a], 7]]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[a -> 1, a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[a -> 1, a]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[{Association[a -> 1, b -> 2], {a -> 3, c -> 4}}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1]], Association[Rule[a, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[{Association[a -> 1, b -> 2], {a -> 3, c -> 4}}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2]], Association[Rule[c, 4]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[{a -> 1, b -> 2}, a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMap[f, Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[f[a], 1], Rule[f[b], 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyMap[f][Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[f[a], 1], Rule[f[b], 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeyMap[f, Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[f[a], Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeyValueMap[f, Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, 1], f[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyValueMap[f][Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, 1], f[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[Association[a -> 1, b -> 2], a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[{a -> 1, b -> 2}, a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[Association[a -> 1, b -> 2], z]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[{a -> 1, b -> 2}, z]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySelect[Association[a -> 1, b -> 2, c -> 3], MatchQ[#, Alternatives[a, c]] &]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySelect[MatchQ[#, Alternatives[a, c]] &][Association[a -> 1, b -> 2, c -> 3]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySelect[{a -> 1, a -> 2, b -> 3}, MatchQ[#, a] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeySelect[Association[a :> (x = x + 1)], True &], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[Association[b -> 2, a -> 1, c -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[List[b -> 2, a -> 1, c -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySort[Association[bb -> 2, a -> 1, ccc -> 3], Function[Less[StringLength[ToString[Slot[1]]], StringLength[ToString[Slot[2]]]]]]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySort[List[bb -> 2, a -> 1], Function[Less[StringLength[ToString[Slot[1]]], StringLength[ToString[Slot[2]]]]]]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {KeySort[Association[a :> (x = x + 1)]], KeySort[Association[a :> (x = x + 1)], OrderedQ[{#1, #2}] &], x}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySortBy[Association[a -> 1, bb -> 2, ccc -> 3], StringLength[ToString[#]] &]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySortBy[List[bb -> 2, a -> 1, ccc -> 3], StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySortBy[StringLength[ToString[#]] &][Association[bb -> 2, a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeySortBy[Association[a :> (x = x + 1)], Identity], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[f, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[a]], Rule[b, f[b]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[f][{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[a]], Rule[b, f[b]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[Identity, Association[a -> 1, b :> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], RuleDelayed[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[(h[#] -> #) &, Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[h[Rule[a, 1]], Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {AssociationMap[Identity, Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    let invalid_association_map = session
        .eval_text("AssociationMap[f, Association[a -> 1, b -> 2]]")
        .unwrap();
    assert_eq!(
        invalid_association_map.value.to_string(),
        "Association[f[Rule[a, 1]], f[Rule[b, 2]]]"
    );
    assert_eq!(
        invalid_association_map
            .messages
            .iter()
            .filter(|message| message.symbol == "AssociationMap" && message.tag == "invrlf")
            .count(),
        2
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/functional_ops/callable.rs.

#[test]
fn operator_wrappers_cover_current_curried_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("OperatorApplied[f][y][x]")
            .unwrap()
            .value
            .to_string(),
        "f[x, y]"
    );
    assert_eq!(
        session
            .eval_text("OperatorApplied[f, 3][a][b, c]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("CurryApplied[3][f][a, b][c]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("CurryApplied[f, {2, 1, 2}][a][b]")
            .unwrap()
            .value
            .to_string(),
        "f[b, a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReverseApplied[f][a, b, c]")
            .unwrap()
            .value
            .to_string(),
        "f[c, b, a]"
    );
    assert_eq!(
        session
            .eval_text("ReverseApplied[2][f][a, b, c]")
            .unwrap()
            .value
            .to_string(),
        "f[b, a, c]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3, 2][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3, {All, 1}][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3, All][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "NearestTo[3, All][List[1, 2, 5]]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/functional_ops/sequence.rs.

#[test]
fn accumulate_blockmap_takewhile_and_fold_family_cover_current_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Accumulate[{1, 2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3, 6]"
    );
    assert_eq!(
        session
            .eval_text("Accumulate[<|a -> 1, b -> 2|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("BlockMap[Apply[Plus], {1, 2, 3, 4}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 7]"
    );
    assert_eq!(
        session
            .eval_text("BlockMap[Identity, {a, b, c}, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("TakeWhile[Positive][{1, 2, -1, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("TakeWhile[f[1, 2, -1, 3], Positive]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("LengthWhile[<|a -> 1, b -> 2, c -> -1|>, Positive]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("List[Context[Select], Select[{1, 2, 3}, OddQ], Select[{1, 2, 3}, OddQ, 1], Select[OddQ][{1, 2, 3}], Select[OddQ, 1][{1, 2, 3}]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"System`\", List[1, 3], List[1], List[1, 3], List[1]]"
    );
    assert_eq!(
        session
            .eval_text("Select[f[1, 2, 3], OddQ, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[1]"
    );
    assert_eq!(
        session
            .eval_text("Select[<|a -> 1, b -> 2, c -> 3|>, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[{2, 4, 5, 7}, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[{2, 4}, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotFound\"]"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[{2, 4}, OddQ, fallback]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[OddQ][{2, 3, 4}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[<|a -> 2, b -> 3|>, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("libs = {{\"ErrorUtil.wl\"}, {\"Parser.wl\"}}; SelectFirst[Range[Length[libs]], StringContainsQ[Last[libs[[#]]], \"Parser\"] &, $Failed]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Fold[f][{a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "f[f[a, b], c]"
    );
    assert_eq!(
        session
            .eval_text("Fold[f, x, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "f[f[x, a], b]"
    );
    assert_eq!(
        session
            .eval_text("FoldList[f, x, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[x, f[x, a], f[f[x, a], b]]"
    );
    assert_eq!(
        session
            .eval_text(
                "FoldPair[Function[List[Plus[Slot[1], Slot[2]], Subtract[Slot[1], Slot[2]]]], 10, {1, 2}]"
            )
            .unwrap()
            .value
            .to_string(),
        "11"
    );
    assert_eq!(
        session
            .eval_text(
                "FoldPair[Function[List[Plus[Slot[1], Slot[2]], Subtract[Slot[1], Slot[2]]]], 10, {1, 2}, Last]"
            )
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("FoldWhile[Subtract, 10, {1, 2, 20}, Positive]")
            .unwrap()
            .value
            .to_string(),
        "-13"
    );
    assert_eq!(
        session
            .eval_text("FoldWhile[Subtract, Positive][10, {1, 2, 20}]")
            .unwrap()
            .value
            .to_string(),
        "-13"
    );
    assert_eq!(
        session
            .eval_text("SequenceFold[Plus, {1}, {2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
    assert_eq!(
        session
            .eval_text("SequenceFoldList[List, {a, b}, {c, d}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, List[a, b, c], List[b, List[a, b, c], d]]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/holding_and_control_failure_messages.rs.

#[test]
fn failure_message_template_properties_cover_current_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"HeldMessageTemplate\", \"Message\", \"MessageName\", \"MessageParameters\", \"MessageTemplate\", \"StyledMessage\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    session.eval_text("delayedParamsSide = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", Association[\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" :> (delayedParamsSide = 1; {a})]][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad a\""
    );
    assert_eq!(
        session
            .eval_text("delayedParamsSide")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    let failure = session
        .eval_request("Failure[\"tag\", Association[\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" :> {InputString[\"param: \"]}]][\"Message\"]")
        .expect_err("delayed Failure MessageParameters should suspend through host input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "param: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming Failure MessageParameters should finish message rendering");
    assert_eq!(response.value.to_string(), "\"bad ok\"");
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad ``\", \"MessageParameters\" -> {x}}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"bad x\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[\"bad `1`\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotApplicable\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\"}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad `1`\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> 1}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Message\", \"MessageTemplate\", \"StyledMessage\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> 1}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"HeldMessageTemplate\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> 1}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"MessageName\"]"
    );
    assert_eq!(
        session
            .eval_text(
                "Failure[\"tag\", {\"MessageTemplate\" -> 1, \"HeldMessageTemplate\" -> 2}][\"HeldMessageTemplate\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Failure[\"tag\", {\"MessageTemplate\" -> 1, \"MessageName\" -> 3}][\"MessageName\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `Name`\", \"MessageParameters\" -> <|\"Name\" -> \"x\"|>}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    session.eval_text("f::named = \"bad `Name`\"").unwrap();
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad arg x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"named\"], \"MessageParameters\" -> <|\"Name\" -> \"x\"|>}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"bad arg x\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[f::argx]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "\"argx\""
    );
    session
        .eval_text("g::argx := StringJoin[InputString[\"msg: \"], \" `1`\"]")
        .unwrap();
    let failure = session
        .eval_request("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[g, \"argx\"], \"MessageParameters\" -> {x}}][\"Message\"]")
        .expect_err("delayed Failure MessageName template should suspend through host input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("bad")
        .expect("resuming Failure MessageName template should finish message rendering");
    assert_eq!(response.value.to_string(), "\"bad x\"");
}

// Moved from crates/rum-runtime/src/runtime/tests/holding_and_control_part5.rs.

#[test]
fn failure_and_enclose_confirm_family_cover_first_pass_error_flow() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FailureQ[Failure[\"oops\", {\"Information\" -> 1}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Enclose[Confirm[$Failed]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Failed, info], \"Information\"]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[2, IntegerQ]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[x, IntegerQ, info], \"Function\"]")
            .unwrap()
            .value
            .to_string(),
        "IntegerQ"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmMatch[2, _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmMatch[x, _Integer, info], \"Pattern\"]")
            .unwrap()
            .value
            .to_string(),
        "Blank[Integer]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[True]]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[False, info], \"Information\"]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );

    session
        .eval_text("f::argx := StringJoin[InputString[\"msg: \"], \" `1`\"]")
        .unwrap();
    let failure = session
        .eval_request("Enclose[Confirm[Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}]], \"Message\"]")
        .expect_err("Enclose string property handler should suspend while rendering Failure MessageName template");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("bad")
        .expect("resuming Enclose property handler should finish rendering Failure message");
    assert_eq!(response.value.to_string(), "\"bad x\"");
}

// Moved from crates/rum-runtime/src/runtime/tests/holding_and_control_part8.rs.

#[test]
fn confirm_failure_wrapping_resumes_delayed_source_failure_message_template() {
    let mut session = core_session();

    session
        .eval_text("g::argx := StringJoin[InputString[\"msg: \"], \" `1`\"]")
        .unwrap();
    let failure = session
        .eval_request("Enclose[Confirm[Failure[\"tag\", {\"MessageTemplate\" -> MessageName[g, \"argx\"], \"MessageParameters\" -> {x}}]], \"MessageTemplate\"]")
        .expect_err("Confirm should suspend while rendering the source Failure message");
    expect_input_prompt(failure, "msg: ");

    let response = session
        .resume_eval_request_with_input("bad")
        .expect("resuming source Failure message rendering should finish Confirm wrapping");
    assert_eq!(response.value.to_string(), "\"bad x\"");

    let failure = session
        .eval_request("Enclose[Confirm[Failure[\"tag\", {\"MessageTemplate\" -> MessageName[g, \"argx\"], \"MessageParameters\" -> {x}}]], \"MessageParameters\"]")
        .expect_err("Confirm should suspend before producing wrapped message parameters");
    expect_input_prompt(failure, "msg: ");

    let response = session
        .resume_eval_request_with_input("ignored")
        .expect("resuming source Failure message rendering should expose wrapped parameters");
    assert_eq!(response.value.to_string(), "List[]");
}

// Moved from crates/rum-runtime/src/runtime/tests/messages.rs.

#[test]
fn message_list_reports_last_top_level_messages() {
    let mut session = core_session();

    session.eval_text("Message[FindFile::notfound]").unwrap();

    assert_eq!(
        session.eval_text("$MessageList").unwrap().value.to_string(),
        "List[HoldForm[FindFile::notfound]]"
    );

    let mut string_session = core_session();
    string_session
        .eval_text("Message[FindFile::notfound]")
        .unwrap();
    assert_eq!(
        string_session
            .eval_text("ToString[$MessageList, InputForm]")
            .unwrap()
            .value
            .to_string(),
        "\"{HoldForm[FindFile::notfound]}\""
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/mma_hol_cold_lane.rs.

#[test]
fn logical_and_comparison_syntax_lowers_to_existing_builtins() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("True && !False")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("False || True && False")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("1 === 1 && 1 =!= 2")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("0 <= 1 < 3").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("\"z\" <> ToString[3]")
            .unwrap()
            .value
            .to_string(),
        "\"z3\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"HOLError\", <|\"tag\" -> \"rule\"|>][[2, \"tag\"]]")
            .unwrap()
            .value
            .to_string(),
        "\"rule\""
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/options_and_protection.rs.

#[test]
fn delayed_system_default_options_feed_current_builtin_consumers() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("SetOptions[StringContainsQ, IgnoreCase :> True]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[IgnoreCase, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("SetOptions[Depth, Heads :> True]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/options_and_protection.rs.

#[test]
fn setoptions_updates_current_system_default_subset_behavior() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("SetOptions[StringContainsQ, IgnoreCase -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\", IgnoreCase -> False]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );

    assert_eq!(
        session
            .eval_text("SetOptions[StringCount, Overlaps -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringCount[\"aaaa\", \"aa\"]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/options_and_protection.rs.

#[test]
fn system_default_options_cover_current_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Options[Cases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[FreeQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Position]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[StringCount]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[CheckAbort]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[PropagateAborts, Automatic]]"
    );
    assert_eq!(
        session
            .eval_text("Options[InputString]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[FieldMasked, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Input]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[FieldMasked, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[ArrayFilter]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Padding, \"Fixed\"]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Save]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExcludedContexts, Automatic], Rule[IncludedContexts, All]]"
    );
    assert_eq!(
        session
            .eval_text("Options[DumpSave]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[CompressionLevel, 1], Rule[SymbolAttributes, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Once]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExpirationDate, None], Rule[PersistenceTime, None]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Names]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[ResolveContextAliases, True], Rule[SpellingCorrection, False]]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/package_tools.rs.

#[test]
fn dump_save_context_strings_roundtrip_public_and_private_contexts() {
    let dir = TempDirGuard::new("dumpsave-contexts");
    let file = dir.path().join("bootstrap.mx");
    let mut session = core_session();

    session
        .eval_script(
            r#"
BeginPackage["Snap`"]
public::usage = "public"
Begin["`Private`"]
helper[x_] := x + 1
End[]
public[x_] := Snap`Private`helper[x]
EndPackage[]
"#,
        )
        .unwrap();
    assert_eq!(
        session
            .eval_text("Snap`public[4]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session
            .eval_text("holContexts = Select[Contexts[], StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\", \"Snap`Private`\"]"
    );

    let dumped = session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], Evaluate[holContexts]]",
            rum_path(&file)
        ))
        .unwrap();
    assert!(dumped.value.to_string().contains("Snap`public"));
    assert!(dumped.value.to_string().contains("Snap`Private`helper"));

    let mut restored = core_session();
    restored
        .eval_text(&format!("Get[File[\"{}\"]]", rum_path(&file)))
        .unwrap();
    assert_eq!(
        restored
            .eval_text("Select[Contexts[], StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\", \"Snap`Private`\"]"
    );
    assert_eq!(
        restored
            .eval_text("Select[$Packages, StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\"]"
    );
    assert_eq!(
        restored
            .eval_text("Select[$ContextPath, StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\"]"
    );
    assert_eq!(
        restored.eval_text("public[9]").unwrap().value.to_string(),
        "10"
    );
    let needs = restored.eval_text("Needs[\"Snap`\"]").unwrap();
    assert_eq!(needs.value.to_string(), "Null");
    assert_eq!(
        restored.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Snap`\"]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/functional_tensor_string_resume.rs.

#[test]
fn resume_eval_request_with_input_can_finish_delayed_string_rules() {
    let mut session = core_session();

    let first = session
        .eval_request("StringReplace[{\"a\", \"a\"}, \"a\" :> InputString[\"replace: \"]]")
        .expect_err("StringReplace delayed rule should suspend on the first string");
    assert_input_prompt(&first.error, "replace: ");

    let second = session
        .resume_eval_request_with_input("x")
        .expect_err("StringReplace should continue to the second string after resuming");
    assert_input_prompt(&second.error, "replace: ");

    let response = session
        .resume_eval_request_with_input("y")
        .expect("StringReplace should finish after resumed delayed rules");
    assert_eq!(response.value.to_string(), "List[\"x\", \"y\"]");

    let first = session
        .eval_request("StringCases[{\"a\", \"b\"}, x_ :> InputString[\"case: \"]]")
        .expect_err("StringCases delayed rule should suspend on the first string");
    assert_input_prompt(&first.error, "case: ");

    let second = session
        .resume_eval_request_with_input("a")
        .expect_err("StringCases should continue to the second string after resuming");
    assert_input_prompt(&second.error, "case: ");

    let response = session
        .resume_eval_request_with_input("b")
        .expect("StringCases should finish after resumed delayed rules");
    assert_eq!(response.value.to_string(), "List[List[\"a\"], List[\"b\"]]");
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/functional_tensor_string_resume.rs.

#[test]
fn resume_eval_request_with_input_can_finish_string_apply_list_callbacks() {
    let mut session = core_session();

    let first = session
        .eval_request("StringApply[Function[List[InputString[\"apply: \"]]], {\"ab\", \"cd\"}]")
        .expect_err("StringApply callback should suspend on the first string");
    assert_input_prompt(&first.error, "apply: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("StringApply should continue to the second string after resuming");
    assert_input_prompt(&second.error, "apply: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("StringApply should finish after resumed callbacks");
    assert_eq!(response.value.to_string(), "List[\"one\", \"two\"]");
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume.rs.

#[test]
fn resume_eval_request_with_input_can_finish_replacepart_pattern_bindings_in_delayed_rhs() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "ReplacePart[f[g[a]], {1, i_} :> StringJoin[InputString[\"rhs: \"], ToString[i]]]",
        )
        .expect_err(
            "ReplacePart should suspend when a delayed rhs with pattern bindings requests input",
        );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue ReplacePart after the delayed rhs resolves");

    assert_eq!(response.value.to_string(), "f[g[\"done1\"]]");
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume_associations.rs.

#[test]
fn resume_eval_request_with_input_can_finish_keysortby_after_suspending_sort_key() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "KeySortBy[{bb -> 2, a -> 1}, Function[If[SameQ[Slot[1], bb], StringLength[InputString[\"key: \"]], StringLength[ToString[Slot[1]]]]]]",
        )
        .expect_err("KeySortBy should suspend when a transformed sort key requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "key: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("xxxx")
        .expect("resume should continue KeySortBy after the transformed sort key resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/structure_ops/dedup.rs.

#[test]
fn delete_duplicates_by_keeps_first_item_for_each_evaluated_key() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("DeleteDuplicatesBy[{1, 2, 3, 4}, Mod[#, 2] &]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("DeleteDuplicatesBy[f[1, 2, 3, 4], Mod[#, 2] &]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("DeleteDuplicatesBy[{a, b, aa, c}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "List[a, aa]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/syntax_information_core_catalog.rs.

#[test]
fn syntax_information_includes_current_system_default_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("SyntaxInformation[Names]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Optional[Blank[]], OptionsPattern]], Rule[\"OptionNames\", List[\"IgnoreCase\", \"ResolveContextAliases\", \"SpellingCorrection\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Options]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[SetOptions]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankNullSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[ArgumentsOptions]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Optional[Blank[Association]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Protect]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankNullSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Unprotect]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[DeclarePackage]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], List[BlankSequence[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Begin]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[BeginPackage]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankNullSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[End]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[EndPackage]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Needs]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[MessageName]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Message]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Module]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[List[BlankSequence[]], Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Block]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[List[BlankSequence[]], Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Function]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Table]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankSequence[]]], Rule[\"LocalVariables\", List[\"Table\", List[2, Infinity]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Do]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankSequence[]]], Rule[\"LocalVariables\", List[\"Table\", List[2, Infinity]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Save]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]], Rule[\"OptionNames\", List[\"ExcludedContexts\", \"IncludedContexts\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[DumpSave]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]], Rule[\"OptionNames\", List[\"CompressionLevel\", \"SymbolAttributes\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Once]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]], Rule[\"OptionNames\", List[\"ExpirationDate\", \"PersistenceTime\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringCount]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringContainsQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringPosition]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringCases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringDelete]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringReplace]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Cases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Count]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[MemberQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[FreeQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Position]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Level]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Depth]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[CheckAbort]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[ArrayFilter]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Blank[], OptionsPattern]]]"
    );
}
