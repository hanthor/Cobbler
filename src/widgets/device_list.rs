use std::ffi::CString;
// use adw::prelude::*;
// use glib::clone;
// use udisks::prelude::*; // Removed invalid import

// Since we are not passing the full AppWindow struct easily due to circular deps if not careful,
// we will simplify the `new` function or just provide the metadata fetching logic.
// The original Impression code passes `ImpressionAppWindow`.

#[derive(Debug, Clone)]
pub struct DeviceMetadata {
    pub object: udisks::Object,
    pub display_string: Option<String>,
    pub info: Option<String>,
    pub label: udisks::Result<String>,
}

pub async fn fetch_devices_metadata() -> udisks::Result<Vec<DeviceMetadata>> {
    let client = udisks::Client::new().await?;
    let devices = refresh_devices(&client).await?;
    Ok(get_devices_metadata(&client, &devices).await)
}

async fn refresh_devices(client: &udisks::Client) -> udisks::Result<Vec<udisks::Object>> {
    let mut drives = vec![];
    for object in client
        .object_manager()
        .get_managed_objects()
        .await?
        .into_iter()
        .filter_map(|(object_path, _)| client.object(object_path).ok())
    {
        // We need check for Drive interface
        let Ok(drive) = object.drive().await else {
            continue;
        };
        
        // Check removable
        if !drive.removable().await.unwrap_or(true) {
            continue;
        }

        if let Some(block) = client.block_for_drive(&drive, false).await {
            let object = client.object(block.inner().path().to_owned()).unwrap();
            drives.push(object);
        }
    }

    drives.sort_unstable_by_key(|x| x.object_path().to_string());

    Ok(drives)
}

async fn get_devices_metadata(
    client: &udisks::Client,
    devices: &[udisks::Object],
) -> Vec<DeviceMetadata> {
    let mut res = Vec::new();
    for device in devices {
        let metadata = device_metadata(client, device).await;
        res.push(metadata);
    }
    res
}

async fn device_metadata(client: &udisks::Client, object: &udisks::Object) -> DeviceMetadata {
    DeviceMetadata {
        object: object.clone(),
        display_string: preferred_device_display_string(object).await,
        info: device_info(client, object).await,
        label: device_label(client, object).await,
    }
}

pub async fn preferred_device_display_string(object: &udisks::Object) -> Option<String> {
    let block = object.block().await.ok()?;
    let preferred_device = block.preferred_device().await.ok()?;
    Some(
        CString::from_vec_with_nul(preferred_device)
            .ok()?
            .to_str()
            .ok()?
            .to_string(),
    )
}

async fn device_info(_client: &udisks::Client, device: &udisks::Object) -> Option<String> {
    // In Impression this uses `object_info`. 
    // Does udisks crate provide `object_info` helper? 
    // Impression might have an extension trait or it is in udisks crate. 
    // Checking Impression source, it calls `client.object_info(device)`. 
    // If it's not standard udisks2 crate, it might be an extension in Impression?
    // Let's assume udisks2 crate has it or we simplify.
    // If it fails to compile, we will fix.
    // For now, let's just return None to be safe or try to implement basic info.
    // udisks crate documentation is sparse in my head usually.
    // Let's look at what `device_label` does.
    
    // Simplification:
    let drive = device.drive().await.ok()?;
    let model = drive.model().await.ok()?;
    let vendor = drive.vendor().await.ok()?;
    Some(format!("{} {}", vendor, model))
}


pub async fn device_label(
    client: &udisks::Client,
    object: &udisks::Object,
) -> udisks::Result<String> {
    let block = object.block().await?;
    let parent_id_label = block.id_label().await.ok();
    let mut partition_id_label = None;

    if let Ok(partition_table) = object.partition_table().await {
        for partition in client
            .partitions(&partition_table)
            .await
            .iter()
            .filter_map(|partition| client.object(partition.inner().path().clone()).ok())
        {
            let Ok(partition) = partition.partition().await else {
                continue;
            };
            partition_id_label = partition.name().await.ok();
            break;
        }
    }
    let drive = client.drive_for_block(&block).await?;
    let vendor = drive.vendor().await?;
    let model = drive.model().await?;
    Ok(parent_id_label.or(partition_id_label).map_or_else(
        || format!("{vendor} {model}").trim().to_owned(),
        |label| format!("{label} ({vendor} {model})").trim().to_owned(),
    ))
}
