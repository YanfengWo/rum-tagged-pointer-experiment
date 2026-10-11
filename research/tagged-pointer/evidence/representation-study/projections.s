	.text
	.file	"rum_representation_study.6710eb3f003fee71-cgu.0"
	.section	.text._ZN24rum_representation_study24inspect_v1_string_length17hf41ae715fdf0405fE,"ax",@progbits
	.globl	_ZN24rum_representation_study24inspect_v1_string_length17hf41ae715fdf0405fE
	.p2align	4, 0x90
	.type	_ZN24rum_representation_study24inspect_v1_string_length17hf41ae715fdf0405fE,@function
_ZN24rum_representation_study24inspect_v1_string_length17hf41ae715fdf0405fE:
	.cfi_startproc
	testb	$1, (%rdi)
	jne	.LBB0_1
	pushq	%rax
	.cfi_def_cfa_offset 16
	callq	*_ZN19rum_compact_expr_v111CompactExpr8heap_ptr17h1ec8aea3c8ff1d05E@GOTPCREL(%rip)
	cmpl	$2, (%rax)
	jne	.LBB0_4
	movq	24(%rax), %rdx
	movl	$1, %eax
	addq	$8, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB0_1:
	xorl	%eax, %eax
	retq
.LBB0_4:
	.cfi_def_cfa_offset 16
	xorl	%eax, %eax
	addq	$8, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	_ZN24rum_representation_study24inspect_v1_string_length17hf41ae715fdf0405fE, .Lfunc_end0-_ZN24rum_representation_study24inspect_v1_string_length17hf41ae715fdf0405fE
	.cfi_endproc

	.section	.text._ZN24rum_representation_study24inspect_v2_string_length17hfcb77a064055746cE,"ax",@progbits
	.globl	_ZN24rum_representation_study24inspect_v2_string_length17hfcb77a064055746cE
	.p2align	4, 0x90
	.type	_ZN24rum_representation_study24inspect_v2_string_length17hfcb77a064055746cE,@function
_ZN24rum_representation_study24inspect_v2_string_length17hfcb77a064055746cE:
	.cfi_startproc
	movq	(%rdi), %rax
	testb	$1, %al
	jne	.LBB1_1
	cmpl	$2, (%rax)
	jne	.LBB1_1
	movq	24(%rax), %rdx
	movl	$1, %eax
	retq
.LBB1_1:
	xorl	%eax, %eax
	retq
.Lfunc_end1:
	.size	_ZN24rum_representation_study24inspect_v2_string_length17hfcb77a064055746cE, .Lfunc_end1-_ZN24rum_representation_study24inspect_v2_string_length17hfcb77a064055746cE
	.cfi_endproc

	.section	.text._ZN24rum_representation_study31inspect_v1_hinted_string_length17h242e56744e959f19E,"ax",@progbits
	.globl	_ZN24rum_representation_study31inspect_v1_hinted_string_length17h242e56744e959f19E
	.p2align	4, 0x90
	.type	_ZN24rum_representation_study31inspect_v1_hinted_string_length17h242e56744e959f19E,@function
_ZN24rum_representation_study31inspect_v1_hinted_string_length17h242e56744e959f19E:
	.cfi_startproc
	movq	(%rdi), %rcx
	movq	%rcx, %rdx
	andq	$-8, %rdx
	movq	%rcx, %rax
	sarq	$3, %rax
	testb	$1, %cl
	cmoveq	%rdx, %rax
	jne	.LBB2_1
	cmpl	$2, (%rax)
	jne	.LBB2_1
	movq	24(%rax), %rdx
	movl	$1, %eax
	retq
.LBB2_1:
	xorl	%eax, %eax
	retq
.Lfunc_end2:
	.size	_ZN24rum_representation_study31inspect_v1_hinted_string_length17h242e56744e959f19E, .Lfunc_end2-_ZN24rum_representation_study31inspect_v1_hinted_string_length17h242e56744e959f19E
	.cfi_endproc

	.ident	"rustc version 1.85.0 (4d91de4e4 2025-02-17)"
	.section	".note.GNU-stack","",@progbits
