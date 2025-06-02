use native_windows_gui::{EmbedResource, Icon};

#[macro_export]
macro_rules! rs {
    ($res_id:ident) => {
        &RESOURCES.with(|r| r.string($res_id))
    };
}

#[macro_export]
macro_rules! r_icon {
    ($res_id:ident) => {
        RESOURCES.with(|r| r.icon($res_id))
    };
}

thread_local! {
    pub(crate) static RESOURCES: Resources = Resources::new();
}

pub(crate) struct Resources {
    embed: EmbedResource,
}

impl Resources {
    fn new() -> Self {
        Self {
            embed: EmbedResource::load(None).expect("Unable to load embedded resources"),
        }
    }

    pub(crate) fn icon(&self, res_id: usize) -> Icon {
        let mut icon = Icon::default();

        Icon::builder()
            .source_embed(Some(&self.embed))
            .source_embed_id(res_id)
            .strict(true)
            .size(Some((16, 16)))
            .build(&mut icon)
            .expect("Unable to load resource icon");

        icon
    }

    pub(crate) fn string(&self, res_id: usize) -> String {
        self.embed
            .string(res_id as u32)
            .expect("Unable to read resource string")
    }
}
