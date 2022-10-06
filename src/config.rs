use crate::global::get_config_dir;
use crate::utils::{Vec3, Vec4};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

trait MergeSerialize {
    type PartialType;
    fn merge(&mut self, parital: Self::PartialType);
}

enum OptionalValue<T: MergeSerialize> {
    Value(T::PartialType),
    Omitted,
}

impl<T: MergeSerialize> OptionalValue<T> {
    pub fn merge_value(self, target: &mut T) {
        if let OptionalValue::Value(partial) = self {
            MergeSerialize::merge(target, partial)
        }
    }
}

impl<T: MergeSerialize> Default for OptionalValue<T> {
    fn default() -> Self {
        OptionalValue::Omitted
    }
}

impl<'de, T: MergeSerialize> serde::Deserialize<'de> for OptionalValue<T>
where
    T::PartialType: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer).map(OptionalValue::Value)
    }
}

macro_rules! merging_serde {
    (
        $(#[$attr: meta])*
        $access: vis struct $name: ident {
            $(
            $(#[$field_attr: meta])*
            $field_vis: vis $field_name: ident: $field_type: ty
            ),*
            $(,)?
        }
        $($rest: tt)*
    ) => {
        $(#[$attr])*
        #[derive(serde::Serialize)]
        $access struct $name {
            $( $(#[$field_attr])* $field_name: $field_type, )*
        }

        #[doc(hidden)]
        const _: () = {
            #[derive(serde::Deserialize)]
            struct Partial {
                $(
                $(#[$field_attr])*
                #[serde(default)]
                $field_vis $field_name: OptionalValue<$field_type>,
                )*
            }

            impl MergeSerialize for $name {
                type PartialType = Partial;

                fn merge(&mut self, parital: Self::PartialType) {
                    $( parital.$field_name.merge_value(&mut self.$field_name); )*
                }
            }
        };

        merging_serde!{ $($rest)* }
    };
    () => {};
}

merging_serde! {
    pub struct OverlayPositionConfig {
        // in degree
        yaw: f32,
        pitch: f32,
        // in meter
        distance: f32,
        // width = distance * witchRadio
        #[serde(rename="widthRadio")]
        width_radio: f32,
        alpha: f32,
    }

    pub struct RingOverlayConfig {
        position: OverlayPositionConfig,
        #[serde(rename="centerColor")]
        center_color: Vec4,
        #[serde(rename="backgroundColor")]
        background_color: Vec4,
        #[serde(rename="edgeColor")]
        edge_color: Vec4,
        #[serde(rename="normalCharColor")]
        normal_char_color: Vec3,
        #[serde(rename="unSelectingCharColor")]
        un_selecting_char_color: Vec3,
        #[serde(rename="selectingCharColor")]
        selecting_char_color: Vec3,
    }

    pub struct CompletionOverlayConfig {
        position: OverlayPositionConfig,
        #[serde(rename="backgroundColor")]
        background_color: Vec3,
        #[serde(rename="inputtingCharColor")]
        inputting_char_color: Vec3,
    }

    pub struct CleKeyConfig {
        #[serde(rename="leftRing")]
        left_ring: RingOverlayConfig,
        #[serde(rename="rightRing")]
        right_ring: RingOverlayConfig,
        completion: CompletionOverlayConfig,
    }
}

//CleKeyConfig loadConfig(CleKeyConfig &config);

fn get_config_path() -> PathBuf {
    return get_config_dir().join("config.json");
}

fn do_load_config(config: &mut CleKeyConfig) -> io::Result<()> {
    let config_path = get_config_path();
    let config_file = File::open(config_path)?;

    CleKeyConfig::merge(config, serde_json::from_reader(config_file)?);
    Ok(())
}

fn write_config(config: &CleKeyConfig) -> io::Result<()> {
    let mut writing = File::create(get_config_path())?;
    serde_json::to_writer(&mut writing, config)?;
    writing.flush()?;
    Ok(())
}

fn load_config(config: &mut CleKeyConfig) {
    if let Err(err) = do_load_config(config) {
        log::error!("loading config: {}", err);
    }
    if let Err(err) = write_config(config) {
        log::error!("saving config: {}", err);
    }
}

////////////////////////////////////////

// the trait to simplify primitive values for MergeSerializable
trait MergeSerializePrimitive {}

impl<T: MergeSerializePrimitive> MergeSerialize for T {
    type PartialType = T;

    #[inline(always)]
    fn merge(&mut self, parital: Self::PartialType) {
        *self = parital;
    }
}

impl MergeSerializePrimitive for u8 {}
impl MergeSerializePrimitive for u16 {}
impl MergeSerializePrimitive for u32 {}
impl MergeSerializePrimitive for u64 {}
impl MergeSerializePrimitive for i8 {}
impl MergeSerializePrimitive for i16 {}
impl MergeSerializePrimitive for i32 {}
impl MergeSerializePrimitive for i64 {}
impl MergeSerializePrimitive for f32 {}
impl MergeSerializePrimitive for f64 {}
impl MergeSerializePrimitive for Vec3 {}
impl MergeSerializePrimitive for Vec4 {}
impl MergeSerializePrimitive for String {}

////////////////////////////////////////
