use crate::controller::start_controller;
use crate::error::Error;
use crate::message::Command;
use crate::meta::{AlgoConfig, AsyncObjectiveFunction};
use crate::result::FinalReport;
use crate::spec::Spec;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::oneshot;
use futures::StreamExt;

pub async fn launch<F: AsyncObjectiveFunction>(
    spec: Spec,
    obj_func: F,
    algo_config: AlgoConfig,
    mut cmd_recv: UnboundedReceiver<Command>,
    max_num_eval: Option<usize>,
    target_obj_func_val: Option<f64>,
) -> Result<FinalReport, Error> {
    let mut abort_sig_sender_holder: Option<oneshot::Sender<()>>;
    let (abort_sig_sender, abort_signal_recv) = oneshot::channel();
    abort_sig_sender_holder = Some(abort_sig_sender);

    let controller = start_controller(
        algo_config,
        spec,
        obj_func,
        abort_signal_recv,
        max_num_eval,
        target_obj_func_val,
    );

    tokio::pin!(controller);

    loop {
        tokio::select! {
            cmd = cmd_recv.next() => {
                if let Some(Command::Terminate) = cmd {
                    abort_sig_sender_holder.take().unwrap().send(()).ok();
                } else {
                    return Err(Error::ClientHungUp);
                }
            }
            res = &mut controller => {
                return res;
            }
        }
    }
}
