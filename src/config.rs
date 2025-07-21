use crate::global::get_appdata_dir;
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use skia_safe::Color4f;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

trait MergeSerialize {
    type PartialType;
    fn merge(&mut self, parital: Self::PartialType);
}

#[derive(Default)]
enum OptionalValue<T: MergeSerialize> {
    Value(T::PartialType),
    #[default]
    Omitted,
}

impl<T: MergeSerialize> OptionalValue<T> {
    pub fn merge_value(self, target: &mut T) {
        if let OptionalValue::Value(partial) = self {
            MergeSerialize::merge(target, partial)
        }
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
        Deserialize::deserialize(deserializer).map(OptionalValue::Value)
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
            $( $(#[$field_attr])* $field_vis $field_name: $field_type, )*
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
    #[derive(Debug)]
    pub struct OverlayPositionConfig {
        // in degree
        pub yaw: f32,
        pub pitch: f32,
        // in meter
        pub distance: f32,
        // width = distance * witchRadio
        #[serde(rename="widthRadio")]
        pub width_radio: f32,
        pub alpha: f32,
    }

    #[derive(Debug)]
    pub struct RingOverlayConfig {
        pub position: OverlayPositionConfig,
        #[serde(rename="centerColor", with="serialize_color4f_3f")]
        pub center_color: Color4f,
        #[serde(rename="backgroundColor", with="serialize_color4f_3f")]
        pub background_color: Color4f,
        #[serde(rename="edgeColor", with="serialize_color4f_3f")]
        pub edge_color: Color4f,
        #[serde(rename="normalCharColor", with="serialize_color4f_3f")]
        pub normal_char_color: Color4f,
        #[serde(rename="unSelectingCharColor", with="serialize_color4f_3f")]
        pub un_selecting_char_color: Color4f,
        #[serde(rename="selectingCharColor", with="serialize_color4f_3f")]
        pub selecting_char_color: Color4f,
        #[serde(rename="selectingCharInRingColor", with="serialize_color4f_3f")]
        pub selecting_char_in_ring_color: Color4f,
    }

    #[derive(Debug)]
    pub struct CompletionOverlayConfig {
        pub position: OverlayPositionConfig,
        #[serde(rename="backgroundColor", with="serialize_color4f_3f")]
        pub background_color: Color4f,
        #[serde(rename="inputtingCharColor", with="serialize_color4f_3f")]
        pub inputting_char_color: Color4f,
    }

    #[derive(Debug)]
    pub struct TwoRingMode {
        #[serde(rename="leftRing")]
        pub left_ring: RingOverlayConfig,
        #[serde(rename="rightRing")]
        pub right_ring: RingOverlayConfig,
        pub completion: CompletionOverlayConfig,
    }

    #[derive(Debug)]
    pub struct OneRingMode {
        pub ring: RingOverlayConfig,
        pub completion: CompletionOverlayConfig,
    }

    #[derive(Debug)]
    pub struct Click {
        pub offset: u128,
        pub length: u128,
    }
}

#[derive(Debug, Serialize)]
pub struct CleKeyConfig {
    #[serde(rename = "uiMode")]
    pub ui_mode: UIMode,
    #[serde(rename = "twoRing")]
    pub two_ring: TwoRingMode,
    #[serde(rename = "oneRing")]
    pub one_ring: OneRingMode,
    pub click: Click,
    pub fps: f32,
    pub always_enter_paste: bool,
    pub always_use_buffer: bool,
}

impl Default for CleKeyConfig {
    fn default() -> Self {
        Self {
            ui_mode: Default::default(),
            two_ring: Default::default(),
            one_ring: Default::default(),
            click: Default::default(),
            fps: 72.0,
            always_enter_paste: false,
            always_use_buffer: true,
        }
    }
}

#[doc(hidden)]
const _: () = {
    #[derive(serde::Deserialize)]
    struct Partial {
        #[serde(rename = "uiMode")]
        #[serde(default)]
        pub ui_mode: OptionalValue<UIMode>,
        #[serde(rename = "twoRing")]
        #[serde(default)]
        pub two_ring: OptionalValue<TwoRingMode>,
        #[serde(rename = "oneRing")]
        #[serde(default)]
        pub one_ring: OptionalValue<OneRingMode>,
        #[serde(default)]
        pub click: OptionalValue<Click>,
        pub fps: OptionalValue<f32>,

        // old config
        #[serde(rename = "leftRing")]
        #[serde(default)]
        pub left_ring: OptionalValue<RingOverlayConfig>,
        #[serde(rename = "rightRing")]
        #[serde(default)]
        pub right_ring: OptionalValue<RingOverlayConfig>,
        #[serde(default)]
        pub completion: OptionalValue<CompletionOverlayConfig>,
        #[serde(default)]
        pub always_enter_paste: OptionalValue<bool>,
        #[serde(default)]
        pub always_use_buffer: OptionalValue<bool>,
    }

    impl MergeSerialize for CleKeyConfig {
        type PartialType = Partial;

        fn merge(&mut self, partial: Self::PartialType) {
            // first parse old config to allow override with new config
            partial.left_ring.merge_value(&mut self.two_ring.left_ring);
            partial
                .right_ring
                .merge_value(&mut self.two_ring.right_ring);
            partial
                .completion
                .merge_value(&mut self.two_ring.completion);

            // then, new config format
            partial.ui_mode.merge_value(&mut self.ui_mode);
            partial.two_ring.merge_value(&mut self.two_ring);
            partial.one_ring.merge_value(&mut self.one_ring);
            partial.click.merge_value(&mut self.click);
            partial.fps.merge_value(&mut self.fps);
            partial
                .always_enter_paste
                .merge_value(&mut self.always_enter_paste);
            partial
                .always_use_buffer
                .merge_value(&mut self.always_use_buffer);
        }
    }
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum UIMode {
    TwoRing,
    #[default]
    OneRing,
}

#[allow(dead_code)]
mod serialize_color4f_4f {
    use super::OptionalValue;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use skia_safe::{Color4f, scalar};

    pub fn serialize<S: Serializer>(value: &Color4f, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(value.as_array(), serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<OptionalValue<Color4f>, D::Error> {
        <[scalar; 4] as Deserialize>::deserialize(deserializer)
            .map(|[r, g, b, a]| Color4f::new(r, g, b, a))
            .map(OptionalValue::Value)
    }
}

mod serialize_color4f_3f {
    use super::OptionalValue;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use skia_safe::{Color4f, scalar};

    pub fn serialize<S: Serializer>(value: &Color4f, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&value.as_array()[..3], serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<OptionalValue<Color4f>, D::Error> {
        <[scalar; 3] as Deserialize>::deserialize(deserializer)
            .map(|[r, g, b]| Color4f::new(r, g, b, 1.0))
            .map(OptionalValue::Value)
    }
}

impl Default for TwoRingMode {
    fn default() -> Self {
        Self {
            left_ring: RingOverlayConfig {
                position: OverlayPositionConfig {
                    yaw: 6.0885,
                    pitch: -18.3379,
                    distance: 0.75,
                    width_radio: 0.2,
                    alpha: 1.0,
                },
                ..Default::default()
            },
            right_ring: RingOverlayConfig {
                position: OverlayPositionConfig {
                    yaw: -6.0885,
                    pitch: -18.3379,
                    distance: 0.75,
                    width_radio: 0.2,
                    alpha: 1.0,
                },
                ..Default::default()
            },
            completion: CompletionOverlayConfig {
                position: OverlayPositionConfig {
                    yaw: 0.0,
                    pitch: -26.565,
                    distance: 0.75,
                    width_radio: 0.333,
                    alpha: 1.0,
                },
                background_color: Color4f::new(0.188, 0.345, 0.749, 1.0),
                inputting_char_color: Color4f::new(1.0, 0.0, 0.0, 1.0),
            },
        }
    }
}

impl Default for OneRingMode {
    fn default() -> Self {
        Self {
            ring: RingOverlayConfig {
                position: OverlayPositionConfig {
                    yaw: 0.0,
                    pitch: -18.3379,
                    distance: 0.75,
                    width_radio: 0.3,
                    alpha: 1.0,
                },
                ..Default::default()
            },
            completion: CompletionOverlayConfig {
                position: OverlayPositionConfig {
                    yaw: 0.0,
                    pitch: -32.0,
                    distance: 0.75,
                    width_radio: 0.333,
                    alpha: 1.0,
                },
                background_color: Color4f::new(0.188, 0.345, 0.749, 1.0),
                inputting_char_color: Color4f::new(1.0, 0.0, 0.0, 1.0),
            },
        }
    }
}

impl Default for Click {
    fn default() -> Self {
        Self {
            offset: 120,
            length: 300,
        }
    }
}

impl Default for RingOverlayConfig {
    fn default() -> Self {
        Self {
            position: OverlayPositionConfig {
                yaw: 0.0,
                pitch: 0.0,
                distance: 0.0,
                width_radio: 0.0,
                alpha: 0.0,
            },
            center_color: Color4f::new(0.83, 0.83, 0.83, 1.0),
            background_color: Color4f::new(0.686, 0.686, 0.686, 1.0),
            edge_color: Color4f::new(1.0, 1.0, 1.0, 1.0),
            normal_char_color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            un_selecting_char_color: Color4f::new(0.5, 0.5, 0.5, 1.0),
            selecting_char_color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            selecting_char_in_ring_color: Color4f::new(1.0, 0.0, 0.0, 1.0),
        }
    }
}

//CleKeyConfig loadConfig(CleKeyConfig &config);

fn get_config_path() -> PathBuf {
    get_appdata_dir().join("config.json")
}

fn do_load_config(config: &mut CleKeyConfig) -> io::Result<()> {
    let config_path = get_config_path();
    let config_file = File::open(config_path)?;

    CleKeyConfig::merge(config, serde_json::from_reader(config_file)?);
    Ok(())
}

fn write_config(config: &CleKeyConfig) -> io::Result<()> {
    fs::create_dir_all(get_config_path().parent().unwrap())?;
    let mut writing = File::create(get_config_path())?;
    serde_json::to_writer_pretty(&mut writing, config)?;
    writing.flush()?;
    Ok(())
}

pub fn load_config(config: &mut CleKeyConfig) {
    if let Err(err) = do_load_config(config) {
        log::error!("loading config: {err}");
    }
    if let Err(err) = write_config(config) {
        log::error!("saving config: {err}");
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
impl MergeSerializePrimitive for u128 {}
impl MergeSerializePrimitive for i8 {}
impl MergeSerializePrimitive for i16 {}
impl MergeSerializePrimitive for i32 {}
impl MergeSerializePrimitive for i64 {}
impl MergeSerializePrimitive for i128 {}
impl MergeSerializePrimitive for f32 {}
impl MergeSerializePrimitive for f64 {}
impl MergeSerializePrimitive for bool {}
impl MergeSerializePrimitive for Vec3 {}
impl MergeSerializePrimitive for Vec4 {}
impl MergeSerializePrimitive for String {}
impl MergeSerializePrimitive for Color4f {}
impl MergeSerializePrimitive for UIMode {}

////////////////////////////////////////
