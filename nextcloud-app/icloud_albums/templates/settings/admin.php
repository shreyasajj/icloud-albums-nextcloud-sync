<?php
/** @var array $_ */
?>

<div class="section" id="icloud-albums-settings">
    <h2><?php p($l->t('iCloud Albums Sync')); ?></h2>

    <p class="settings-hint">
        <?php p($l->t('Configure the connection to the iCloud Albums sync service.')); ?>
    </p>

    <div class="icloud-albums-setting">
        <label for="sync-service-url"><?php p($l->t('Sync Service URL')); ?></label>
        <input
            type="text"
            id="sync-service-url"
            name="sync_service_url"
            value="<?php p($_['sync_service_url']); ?>"
            placeholder="http://localhost:8080"
        />
        <button id="save-sync-service-url"><?php p($l->t('Save')); ?></button>
        <span class="msg"></span>
    </div>
</div>

<script>
document.getElementById('save-sync-service-url').addEventListener('click', function() {
    const url = document.getElementById('sync-service-url').value;
    const msgEl = document.querySelector('.msg');

    OC.AppConfig.setValue('icloud_albums', 'sync_service_url', url, {
        success: function() {
            msgEl.textContent = '✓ Saved';
            msgEl.style.color = 'green';
            setTimeout(() => msgEl.textContent = '', 3000);
        },
        error: function() {
            msgEl.textContent = '✗ Error saving';
            msgEl.style.color = 'red';
        }
    });
});
</script>

<style>
#icloud-albums-settings .icloud-albums-setting {
    margin: 15px 0;
}

#icloud-albums-settings label {
    display: inline-block;
    width: 200px;
    font-weight: bold;
}

#icloud-albums-settings input[type="text"] {
    width: 300px;
    padding: 6px;
}

#icloud-albums-settings button {
    margin-left: 10px;
}

#icloud-albums-settings .msg {
    margin-left: 10px;
}
</style>
