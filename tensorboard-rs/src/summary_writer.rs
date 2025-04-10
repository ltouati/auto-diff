#![allow(clippy::too_many_arguments)]
use crate::event_file_writer::EventFileWriter;
use crate::hparams::{
    hparams, hparams_config, status_config, GenericValue, HyperParameter, Metric,
};
use crate::summary::{histogram_raw, image, scalar};
use protobuf::Message;
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tensorboard_proto::event::{Event, TaggedRunMetadata};
use tensorboard_proto::graph::GraphDef;
use tensorboard_proto::node_def::NodeDef;
//use tensorboard_proto::attr_value::{AttrValue, };
//use tensorboard_proto::tensor_shape::{TensorShapeProto, };
use crate::status::JobStatus;
use tensorboard_proto::step_stats::RunMetadata;
use tensorboard_proto::summary::Summary;
use tensorboard_proto::versions::VersionDef;

pub struct FileWriter {
    writer: EventFileWriter,
}

impl FileWriter {
    pub(crate) fn add_global_summary(&mut self, summary: Summary) {
        let mut evn = Event::new();
        evn.set_summary(summary);
        self.writer.add_event(&evn);
    }
}

impl FileWriter {
    pub fn new<P: AsRef<Path>>(logdir: P) -> FileWriter {
        FileWriter {
            writer: EventFileWriter::new(logdir),
        }
    }
    pub fn get_logdir(&self) -> PathBuf {
        self.writer.get_logdir()
    }
    pub fn add_event(&mut self, event: &Event, step: usize) {
        let mut event = event.clone();

        let mut time_full = 0.0;
        if let Ok(n) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            time_full = n.as_secs_f64();
        }
        event.set_wall_time(time_full);

        event.set_step(step as i64);

        self.writer.add_event(&event)
    }
    pub fn add_summary(&mut self, summary: Summary, step: usize) {
        let mut evn = Event::new();
        evn.set_summary(summary);
        self.add_event(&evn, step)
    }
    pub fn add_graph(&mut self, graph: GraphDef, meta: RunMetadata) {
        let mut graph_vec: Vec<u8> = Vec::new();
        graph.write_to_vec(&mut graph_vec).expect("");
        let mut graph_evn = Event::new();
        graph_evn.set_graph_def(graph_vec);
        self.writer.add_event(&graph_evn);

        let mut meta_vec: Vec<u8> = Vec::new();
        meta.write_to_vec(&mut meta_vec).expect("");
        let mut tagged_meta = TaggedRunMetadata::new();
        tagged_meta.set_tag("profiler".to_string());
        tagged_meta.set_run_metadata(meta_vec);
        let mut meta_evn = Event::new();
        meta_evn.set_tagged_run_metadata(tagged_meta);
        self.writer.add_event(&meta_evn);
    }
    pub fn flush(&mut self) {
        self.writer.flush()
    }
}

pub struct SummaryWriter {
    writer: FileWriter,
    all_writers: HashMap<PathBuf, FileWriter>,
}

impl SummaryWriter {
    pub fn add_job_status(&mut self, job_status: &JobStatus, end_time_secs: Option<i64>) {
        self.writer
            .add_global_summary(status_config(job_status, end_time_secs))
    }
    pub fn new<P: AsRef<Path>>(logdir: P) -> SummaryWriter {
        SummaryWriter {
            writer: FileWriter::new(logdir),
            all_writers: HashMap::new(),
        }
    }
    pub fn add_hparams_config(&mut self, hyper_parameters: &[HyperParameter], metrics: &[Metric]) {
        self.writer
            .add_global_summary(hparams_config(hyper_parameters, metrics));
    }
    pub fn add_hparams(
        &mut self,
        hyper_parameters: HashMap<String, GenericValue>,
        trial_id: Option<String>,
        start_time_secs: Option<i64>,
    ) {
        self.writer
            .add_global_summary(hparams(hyper_parameters, trial_id, start_time_secs));
    }
    pub fn add_scalar(&mut self, tag: &str, scalar_value: f32, step: usize) {
        self.writer.add_summary(scalar(tag, scalar_value), step);
    }
    pub fn add_scalars(&mut self, main_tag: &str, tag_scalar: &HashMap<String, f32>, step: usize) {
        let base_logdir = self.writer.get_logdir();
        for (tag, scalar_value) in tag_scalar.iter() {
            let fw_tag = base_logdir.join(main_tag).join(tag);
            if !self.all_writers.contains_key(&fw_tag) {
                let new_writer = FileWriter::new(fw_tag.clone());
                self.all_writers.insert(fw_tag.clone(), new_writer);
            }
            let fw = self.all_writers.get_mut(&fw_tag).expect("");
            fw.add_summary(scalar(main_tag, *scalar_value), step);
        }
    }

    pub fn export_scalars_to_json(&self) {
        unimplemented!();
    }
    pub fn add_histogram(&mut self) {
        unimplemented!();
    }
    pub fn add_histogram_raw(
        &mut self,
        tag: &str,
        min: f64,
        max: f64,
        num: f64,
        sum: f64,
        sum_squares: f64,
        bucket_limits: &[f64],
        bucket_counts: &[f64],
        step: usize,
    ) {
        if bucket_limits.len() != bucket_counts.len() {
            panic!("bucket_limits.len() != bucket_counts.len()");
        }

        self.writer.add_summary(
            histogram_raw(
                tag,
                min,
                max,
                num,
                sum,
                sum_squares,
                bucket_limits,
                bucket_counts,
            ),
            step,
        );
    }
    pub fn add_image(&mut self, tag: &str, data: &[u8], dim: &[usize], step: usize) {
        self.writer.add_summary(image(tag, data, dim), step);
    }
    pub fn add_images(&mut self) {
        unimplemented!();
    }
    pub fn add_image_with_boxes(&mut self) {
        unimplemented!();
    }
    pub fn add_figure(&mut self) {
        unimplemented!();
    }
    pub fn add_video(&mut self) {
        unimplemented!();
    }
    pub fn add_audio(&mut self) {
        unimplemented!();
    }
    pub fn add_text(&mut self) {
        unimplemented!();
    }
    pub fn add_onnx_graph(&mut self) {
        unimplemented!();
    }
    pub fn add_openvino_graph(&mut self) {
        unimplemented!();
    }
    pub fn add_graph(&mut self, node_list: &[NodeDef]) {
        let mut graph = GraphDef::new();

        let nodes = RepeatedField::from(node_list.to_vec());
        graph.set_node(nodes);

        let mut version = VersionDef::new();
        version.set_producer(22);
        graph.set_versions(version);

        let stats = RunMetadata::new();

        self.writer.add_graph(graph, stats);
    }
    pub fn add_embedding(&mut self) {
        unimplemented!();
    }
    pub fn add_pr_curve(&mut self) {
        unimplemented!();
    }
    pub fn add_pr_curve_raw(&mut self) {
        unimplemented!();
    }
    pub fn add_custom_scalars_multilinechart(&mut self) {
        unimplemented!();
    }
    pub fn add_custom_scalars_marginchart(&mut self) {
        unimplemented!();
    }
    pub fn add_custom_scalars(&mut self) {
        unimplemented!();
    }
    pub fn add_mesh(&mut self) {
        unimplemented!();
    }

    pub fn flush(&mut self) {
        self.writer.flush();
    }
}
