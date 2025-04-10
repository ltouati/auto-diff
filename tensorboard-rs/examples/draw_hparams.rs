use std::collections::HashMap;
use tensorboard_rs::hparams::{GenericValue, HyperParameter, Metric};
use tensorboard_rs::status::JobStatus;
use tensorboard_rs::summary_writer::SummaryWriter;

pub fn main() {
    for i in 0..10 {
        let path = format!("./logdir/test-{:}", i);
        let mut writer = SummaryWriter::new(path);
        let options = ["A", "B", "C", "D"];
        let hparams = vec![
            HyperParameter::new("M1"),
            HyperParameter::with_string("M2", "A Metric"),
            HyperParameter::with_bool("M3", true),
            HyperParameter::with_f64s("M4", &[1f64, 2f64]),
            HyperParameter::with_strings("M5", &options),
        ];
        let metrics = vec![Metric::new("Test Metric")];
        writer.add_hparams_config(&hparams, &metrics);

        let mut data = HashMap::new();
        data.insert("M1".to_string(), GenericValue::Number(i as f64));
        data.insert(
            "M2".to_string(),
            GenericValue::String("A Metric".to_string()),
        );
        data.insert("M3".to_string(), GenericValue::Bool(true));
        data.insert("M4".to_string(), GenericValue::Number((i % 2) as f64));
        data.insert(
            "M5".to_string(),
            GenericValue::String(options[i % 4].to_string()),
        );
        writer.add_hparams(data, Some(format!("test-{:}", i).to_string()), Some(0));

        writer.add_scalar("Test Metric", 0f32, 0);
        writer.add_scalar("Test Metric", 1f32, 1);
        writer.add_scalar("Test Metric", 2f32, 2);

        writer.add_job_status(&JobStatus::Success, None);

        writer.flush();
    }
}
