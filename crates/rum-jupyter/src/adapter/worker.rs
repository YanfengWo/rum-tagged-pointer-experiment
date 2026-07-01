use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    thread::{self, JoinHandle},
};

use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostEvalRequest, HostEvalSuccess, HostFailure,
    HostFailureCode, HostSession,
};

use crate::{KernelExecuteOutcome, KernelExecuteRequest, KernelMessageHeader};

#[derive(Clone, Debug)]
pub(super) struct ExecuteContext {
    pub(super) parent_header: KernelMessageHeader,
    pub(super) request: KernelExecuteRequest,
    pub(super) execution_count: Option<u64>,
}

pub(super) enum ExecuteState {
    Idle,
    Running(ExecuteWorker),
}

pub(super) struct ExecuteWorker {
    handle: JoinHandle<(HostSession, KernelExecuteOutcome)>,
    cancel: HostCancelHandle,
    context: ExecuteContext,
    interrupted: bool,
}

enum ExecuteSegment {
    Eval { code: String },
    Resume { input: String },
}

impl ExecuteState {
    pub(super) const fn idle() -> Self {
        Self::Idle
    }

    pub(super) fn is_running(&self) -> bool {
        matches!(self, Self::Running(_))
    }

    pub(super) fn cancel(&mut self) -> bool {
        let Self::Running(worker) = self else {
            return false;
        };
        worker.interrupted = true;
        worker.cancel.cancel();
        true
    }

    pub(super) fn start_eval(&mut self, session: HostSession, context: ExecuteContext) {
        debug_assert!(!self.is_running());
        *self = Self::Running(ExecuteWorker::spawn_eval(session, context));
    }

    pub(super) fn start_resume(
        &mut self,
        session: HostSession,
        context: ExecuteContext,
        input: String,
    ) {
        debug_assert!(!self.is_running());
        *self = Self::Running(ExecuteWorker::spawn_resume(session, context, input));
    }

    pub(super) fn take_finished(&mut self) -> Option<ExecuteWorker> {
        let Self::Running(worker) = self else {
            return None;
        };
        if !worker.handle.is_finished() {
            return None;
        }

        let Self::Running(worker) = std::mem::replace(self, Self::Idle) else {
            unreachable!("checked running state before replace");
        };
        Some(worker)
    }
}

impl ExecuteWorker {
    fn spawn_eval(session: HostSession, context: ExecuteContext) -> Self {
        let cancel = session.cancel_handle();
        let code = context.request.code.clone();
        Self {
            handle: spawn_worker(session, ExecuteSegment::Eval { code }),
            cancel,
            context,
            interrupted: false,
        }
    }

    fn spawn_resume(session: HostSession, context: ExecuteContext, input: String) -> Self {
        let cancel = session.cancel_handle();
        Self {
            handle: spawn_worker(session, ExecuteSegment::Resume { input }),
            cancel,
            context,
            interrupted: false,
        }
    }

    pub(super) fn join(self) -> (ExecuteContext, HostSession, KernelExecuteOutcome, bool) {
        let (session, outcome) = self
            .handle
            .join()
            .expect("execute worker catches eval/resume panics and returns its session");
        (self.context, session, outcome, self.interrupted)
    }
}

pub(super) fn host_outcome_to_kernel(outcome: HostEvalOutcome) -> KernelExecuteOutcome {
    match outcome {
        HostEvalOutcome::Success(response) if is_cancelled_success(&response) => {
            KernelExecuteOutcome::Failure(cancelled_failure_from_success(response))
        }
        HostEvalOutcome::Success(response) => KernelExecuteOutcome::Success(response),
        HostEvalOutcome::InputRequired(suspension) => {
            KernelExecuteOutcome::InputRequired(suspension)
        }
        HostEvalOutcome::Failure(failure) => KernelExecuteOutcome::Failure(failure),
    }
}

fn is_cancelled_success(response: &HostEvalSuccess) -> bool {
    response.value.text == "TerminatedEvaluation[\"Cancelled\"]"
}

fn cancelled_failure_from_success(response: HostEvalSuccess) -> HostFailure {
    HostFailure {
        code: HostFailureCode::Cancelled,
        message: "evaluation cancelled".to_string(),
        messages: response.messages,
        prints: response.prints,
        trace: response.trace,
        outputs: response.outputs,
    }
}

pub(super) fn outcome_requires_session_reset(outcome: &KernelExecuteOutcome) -> bool {
    matches!(
        outcome,
        KernelExecuteOutcome::Failure(failure)
            if matches!(failure.code, HostFailureCode::Panic | HostFailureCode::Poisoned)
    )
}

pub(super) fn outcome_is_cancelled(outcome: &KernelExecuteOutcome) -> bool {
    matches!(
        outcome,
        KernelExecuteOutcome::Failure(failure) if failure.code == HostFailureCode::Cancelled
    )
}

pub(super) fn force_cancelled_success(outcome: KernelExecuteOutcome) -> KernelExecuteOutcome {
    match outcome {
        KernelExecuteOutcome::Success(response) => {
            KernelExecuteOutcome::Failure(cancelled_failure_from_success(response))
        }
        other => other,
    }
}

fn spawn_worker(
    mut session: HostSession,
    segment: ExecuteSegment,
) -> JoinHandle<(HostSession, KernelExecuteOutcome)> {
    thread::spawn(move || {
        let outcome = match catch_unwind(AssertUnwindSafe(|| run_segment(&mut session, segment))) {
            Ok(outcome) => outcome,
            Err(payload) => KernelExecuteOutcome::Failure(worker_panic_failure(&*payload)),
        };
        (session, outcome)
    })
}

fn run_segment(session: &mut HostSession, segment: ExecuteSegment) -> KernelExecuteOutcome {
    let outcome = match segment {
        ExecuteSegment::Eval { code } => session.eval(HostEvalRequest::new(code)),
        ExecuteSegment::Resume { input } => session.resume_with_input(input),
    };
    host_outcome_to_kernel(outcome)
}

fn worker_panic_failure(payload: &(dyn Any + Send)) -> HostFailure {
    HostFailure {
        code: HostFailureCode::Panic,
        message: format!(
            "jupyter execute worker caught panic: {}",
            panic_payload_message(payload)
        ),
        messages: Vec::new(),
        prints: Vec::new(),
        trace: Vec::new(),
        outputs: Vec::new(),
    }
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic".to_string()
    }
}
