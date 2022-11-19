use rumqttc::QoS;

pub const DEFAULT_THROUGHPUT_TOPIC: &str = "THROUGHPUT";
pub const DEFAULT_PING_TOPIC: &str = "PING";
pub const DEFAULT_PONG_TOPIC: &str = "PONG";
pub const DEFAULT_GROUP_ID: &str = "DUMMY_GROUP";
pub const DEFAULT_QOS: QoS = QoS::AtLeastOnce;
pub const DEFAULT_CAP_SIZE: usize = 10;

// pub struct AsyncStdRuntime;

// impl AsyncRuntime for AsyncStdRuntime {
//     type Delay = Pin<Box<dyn Future<Output = ()> + Send>>;

//     fn spawn<T>(task: T)
//     where
//         T: Future<Output = ()> + Send + 'static,
//     {
//         async_std::task::spawn(task);
//     }

//     fn delay_for(duration: Duration) -> Self::Delay {
//         Box::pin(async_std::task::sleep(duration))
//     }
// }

// pub type AsyncStdStreamConsumer = StreamConsumer<DefaultConsumerContext, AsyncStdRuntime>;
// pub type AsyncStdFutureProducer = FutureProducer<DefaultClientContext, AsyncStdRuntime>;

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct KeyVal {
//     pub key: String,
//     pub val: String,
// }

// impl KeyVal {
//     pub fn new<K, V>(key: K, val: V) -> Self
//     where
//         K: ToString,
//         V: ToString,
//     {
//         Self {
//             key: key.to_string(),
//             val: val.to_string(),
//         }
//     }
// }

// impl FromStr for KeyVal {
//     type Err = anyhow::Error;

//     fn from_str(text: &str) -> Result<Self, Self::Err> {
//         let (key, val) = text
//             .split_once('=')
//             .ok_or_else(|| anyhow!("Expect 'KEY=VALUE' string, but get {}", text))?;
//         Ok(Self {
//             key: key.to_string(),
//             val: val.to_string(),
//         })
//     }
// }

// impl fmt::Display for KeyVal {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}={}", self.key, self.val)
//     }
// }

// pub trait MqttErrorExt {
//     fn to_anyhow(self) -> anyhow::Error;
// }

// impl MqttErrorExt for ConnectError {
//     fn to_anyhow(self) -> anyhow::Error {
//         anyhow!("{}", self.to_string())
//     }
// }

// impl MqttErrorExt for ClientError {
//     fn to_anyhow(self) -> anyhow::Error {
//         anyhow!("{}", self.to_string())
//     }
// }
