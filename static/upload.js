// Async file uploader with progress — replaces the old qq-upload library.
// Attaches to any form with [data-upload], POSTs via XHR, shows progress,
// then redirects to the new track page on success.

(function() {
    document.addEventListener('DOMContentLoaded', function() {
        var forms = document.querySelectorAll('form[data-upload]');
        for (var i = 0; i < forms.length; i++) {
            initUploadForm(forms[i]);
        }
    });

    function initUploadForm(form) {
        var fileInput = form.querySelector('input[type="file"]');
        var submitBtn = form.querySelector('input[type="submit"]');
        if (!fileInput || !submitBtn) return;

        // Create progress UI (hidden until upload starts)
        var progress = document.createElement('div');
        progress.className = 'upload-progress';
        progress.style.display = 'none';
        progress.innerHTML =
            '<div class="upload-bar-wrap">' +
                '<div class="upload-bar"></div>' +
            '</div>' +
            '<span class="upload-status">Uploading…</span>';
        form.parentNode.insertBefore(progress, form.nextSibling);

        var bar = progress.querySelector('.upload-bar');
        var status = progress.querySelector('.upload-status');

        form.addEventListener('submit', function(e) {
            e.preventDefault();

            var file = fileInput.files[0];
            if (!file) return;

            var data = new FormData();
            data.append('qqfile', file, file.name);

            var xhr = new XMLHttpRequest();
            xhr.open('POST', form.action, true);
            xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');

            // Show progress, hide form
            form.style.display = 'none';
            progress.style.display = '';

            xhr.upload.addEventListener('progress', function(ev) {
                if (ev.lengthComputable) {
                    var pct = Math.round(ev.loaded / ev.total * 100);
                    bar.style.width = pct + '%';
                    status.textContent = 'Uploading… ' + pct + '%';
                }
            });

            xhr.addEventListener('load', function() {
                if (xhr.status >= 200 && xhr.status < 300) {
                    try {
                        var resp = JSON.parse(xhr.responseText);
                        if (resp.success && resp.tid) {
                            status.textContent = 'Upload complete. Redirecting…';
                            bar.style.width = '100%';
                            window.location.href = '/track/' + resp.tid;
                            return;
                        }
                    } catch(ex) {}
                }
                status.textContent = 'Upload failed.';
                form.style.display = '';
            });

            xhr.addEventListener('error', function() {
                status.textContent = 'Upload failed (network error).';
                form.style.display = '';
            });

            xhr.send(data);
        });
    }
})();
