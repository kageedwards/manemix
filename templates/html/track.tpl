{% import "html/macros.tpl" as m %}

<div class="track">
    <h2>{{ track.title }}
        <span class="buttons">
            {% if logged_in | default(value=false) %}
                {% if not is_favorite | default(value=false) %}
                <form action="/track/{{ track.tid }}/favorite" method="post" style="display:inline;">
                    <input name="nonce" type="hidden" value="{{ nonce }}" />
                    <button type="submit" title="Add to favorites" style="border:none;background:none;cursor:pointer;">
                        <img src="/static/icons/star-empty.png" alt="Add to favorites" />
                    </button>
                </form>
                {% else %}
                <form action="/track/{{ track.tid }}/unfavorite" method="post" style="display:inline;">
                    <input name="nonce" type="hidden" value="{{ nonce }}" />
                    <button type="submit" title="Remove from favorites" style="border:none;background:none;cursor:pointer;">
                        <img src="/static/icons/star.png" alt="Remove from favorites" />
                    </button>
                </form>
                {% endif %}
                {% if track.is_visible and has_playlists | default(value=false) %}
                <span id="addplaylist">
                    <img src="/static/icons/playlist-add.png" alt="Playlists">
                    <form action="/track/{{ track.tid }}/playlist" method="post" title="Add to playlist">
                        <input name="nonce" type="hidden" value="{{ nonce }}"/>
                        <select name="playlist">
                            {% for p in user_playlists %}
                            <option value="{{ p.playlist_id }}">{{ p.playlist_name }}</option>
                            {% endfor %}
                        </select><input type="submit" value="Add to this playlist"/>
                    </form>
                </span>
                {% endif %}
            {% else %}
                <a href="/login?redirect=/track/{{ track.tid }}" title="Add to favorites">
                    <img src="/static/icons/star-empty.png" alt="Add to favorites" />
                </a>
            {% endif %}
        </span>
    </h2>
    <h4>by <a href="/user/{{ track.uid }}">{{ track.username }}</a> <span class="date">on {{ track.day }}</span></h4>

    {% if track.has_art %}<img class="art" alt="" src="/track/{{ track.tid }}/art/medium" />{% endif %}

    <div id="track-media">
    {% if audio.ready %}
    {{ m::player(tid=track.tid, title=track.title, username=track.username, list=track.tid, count=1) }}
    {{ m::player_fallback(tid=track.tid) }}
    {% endif %}
    {% if audio.has_status %}<div class="status" id="transcode-status">Status: {{ audio.status }}</div>{% endif %}

    <div class="toolbar" id="track-toolbar" {% if not audio.ready %}style="display:none;"{% endif %}>
        {% if audio.ready %}
        <span><img alt="" src="/static/icons/drive-download.png" /> Download:
        <ul class="downloads">
            {% if audio.is_mp3_source %}<li><a href="/track/{{ track.tid }}/original"><span>Original MP3
                <span class="codecinfo">Highest quality</span>
            </span></a></li>{% endif %}
            {% if audio.is_other_source %}
            <li><a href="/track/{{ track.tid }}/original"><span>Original ({{ audio.extension }})
                <span class="codecinfo">Highest quality</span>
                <span class="settings">(Untouched upload)</span>
            </span></a></li>
            <li><a href="/track/{{ track.tid }}/mp3"><span>MP3
                <span class="codecinfo">Great quality, widely supported</span>
                <span class="settings">(libmp3lame VBR 0)</span>
            </span></a></li>
            {% endif %}
            <li><a href="/track/{{ track.tid }}/opus"><span>Opus
                <span class="codecinfo">Great quality, small filesize, experimental</span>
                <span class="settings">(libopus ABR ~128kbps)</span>
            </span></a></li>
            <li><a href="/track/{{ track.tid }}/vorbis"><span>OGG Vorbis
                <span class="codecinfo">Good quality, small filesize</span>
                <span class="settings">(libvorbis VBR 4)</span>
            </span></a></li>
            <li><a href="/track/{{ track.tid }}/aac"><span>AAC
                <span class="codecinfo">Okay quality, small filesize</span>
                <span class="settings">(libfdk_aac VBR 3)</span>
            </span></a></li>
            {% if track.has_art %}<li><a href="/track/{{ track.tid }}/art" target="_blank"><span>Cover art</span></a></li>{% endif %}
        </ul>
        </span>
        {% endif %}
        <span><img alt="" src="/static/icons/balloon-white-left.png" /> Share: <a href="#embedcode" onclick="document.getElementById('embedcode').style.display='block';return false;">Embed</a></span>
        {% if not is_owner | default(value=false) %}
        <form action="/track/{{ track.tid }}/report" method="post" style="display:inline;">
            <button type="submit" class="report"><img alt="" src="/static/icons/flag.png" /> <span>Report</span></button>
        </form>
        {% endif %}
    </div>
    </div>

    {% if not audio.ready %}
    <script>
    (function(){
        var tid = {{ track.tid }};
        var poll = setInterval(function(){
            fetch('/track/' + tid + '/status')
                .then(function(r){ return r.json(); })
                .then(function(s){
                    var el = document.getElementById('transcode-status');
                    if (!s.ready) {
                        if (el) el.textContent = 'Status: ' + s.status;
                        return;
                    }
                    clearInterval(poll);
                    // Build player
                    var media = document.getElementById('track-media');
                    var pid = tid + '-1';
                    var pdiv = document.createElement('div');
                    pdiv.id = pid;
                    pdiv.className = 'player';
                    if (el) el.replaceWith(pdiv);
                    else media.insertBefore(pdiv, media.firstChild);
                    var t = {
                        id: pid, tid: '' + tid,
                        title: '{{ track.title | escape }}',
                        artist: '{{ track.username | escape }}',
                        mp3: '/track/' + tid + '/mp3?stream=1',
                        vorbis: '/track/' + tid + '/vorbis?stream=1',
                        aac: '/track/' + tid + '/aac?stream=1',
                        opus: '/track/' + tid + '/opus?stream=1',
                        list: '' + tid
                    };
                    registerTrack(t);
                    initTrack(t);
                    // Build download links
                    var dl = '<span><img alt="" src="/static/icons/drive-download.png" /> Download:' +
                        '<ul class="downloads">';
                    if (s.is_mp3_source) {
                        dl += '<li><a href="/track/' + tid + '/original"><span>Original MP3 <span class="codecinfo">Highest quality</span></span></a></li>';
                    }
                    if (s.is_other_source) {
                        dl += '<li><a href="/track/' + tid + '/original"><span>Original (' + s.extension + ') <span class="codecinfo">Highest quality</span></span></a></li>';
                        dl += '<li><a href="/track/' + tid + '/mp3"><span>MP3 <span class="codecinfo">Great quality, widely supported</span></span></a></li>';
                    }
                    dl += '<li><a href="/track/' + tid + '/opus"><span>Opus <span class="codecinfo">Great quality, small filesize</span></span></a></li>';
                    dl += '<li><a href="/track/' + tid + '/vorbis"><span>OGG Vorbis <span class="codecinfo">Good quality, small filesize</span></span></a></li>';
                    dl += '<li><a href="/track/' + tid + '/aac"><span>AAC <span class="codecinfo">Okay quality, small filesize</span></span></a></li>';
                    dl += '</ul></span>';
                    // Swap in toolbar
                    var toolbar = document.getElementById('track-toolbar');
                    toolbar.insertAdjacentHTML('afterbegin', dl);
                    toolbar.style.display = '';
                });
        }, 3000);
    })();
    </script>
    {% endif %}
    <textarea id="embedcode" style="display:none;">{{ m::embed_code(manemix_url=manemix_url, tid=track.tid, title=track.title, uid=track.uid, username=track.username, width=150) }}</textarea>

    {% if track.has_tags %}
    <div class="toolbar tags"><img alt="" src="/static/icons/tag.png" /> Tags:
        {% for tag in track.tags %}<a href="/tracks/tag/{{ tag }}">{{ tag }}</a>{% endfor %}
    </div>
    {% endif %}

    {{ m::license_badge(track=track) }}
    {% if is_owner | default(value=false) %}<a href="/track/{{ track.tid }}/license">(change)</a>{% endif %}

    {% if track.has_notes %}<div class="notes">{{ track.notes_html | safe }}</div>{% endif %}

    {% if is_owner | default(value=false) %}
    <div id="track-edit" class="dialog">
        <h3 id="track-edit-title"><img src="/static/icons/pencil.png" alt="" /> Edit</h3>

        {% if track.is_hidden %}
        <h4 class="edit-category">Publish</h4>
        <div class="edit-container">
            <div>
                <form class="publish" action="/track/{{ track.tid }}/publish" method="post">
                    <img src="/static/icons/disc-arrow.png" alt="" />
                    This track is not yet published.
                    <input type="submit" value="Publish"/>
                    <input type="hidden" name="tid" value="{{ track.tid }}"/>
                    <input name="nonce" type="hidden" value="{{ nonce }}"/>
                </form>
            </div>
        </div>
        {% endif %}

        <h4 class="edit-category">General</h4>
        <div class="edit-container">
            <div>
                <h4><img alt="" src="/static/icons/rename.png" /> Rename</h4>
                <form method="post" action="/track/{{ track.tid }}/rename">
                    <b>{{ track.username }}</b> -
                    <input type="text" name="title" value="{{ track.title }}" />
                    <input type="submit" value="Rename" />
                    <input name="nonce" type="hidden" value="{{ nonce }}"/>
                </form>
            </div>
            <div>
                <h4><img alt="" src="/static/icons/tag.png" /> Tags</h4>
                <form action="/track/{{ track.tid }}/tags" method="post">
                    <input name="tags" value="{{ track.tags | join(sep=', ') }}" />
                    <input type="submit" value="Update" />
                    <br/>
                    <span class="legend">(comma-separated, e.g. instrumental, electronic)</span>
                    <input name="nonce" type="hidden" value="{{ nonce }}"/>
                </form>
            </div>
            <div>
                <h4><img src="/static/icons/picture.png" alt="" /> Art</h4>
                <form action="/track/{{ track.tid }}/art/upload" method="post" enctype="multipart/form-data">
                    <input type="file" name="file" />
                    <br />
                    {% if track.has_art %}
                    <form action="/track/{{ track.tid }}/art/delete" method="post" style="display:inline;">
                        <button type="submit" class="fakelink danger">(Delete existing cover art)</button>
                        <input name="nonce" type="hidden" value="{{ nonce }}"/>
                    </form>
                    {% endif %}
                    <input type="submit" value="Upload cover art" />
                </form>
            </div>
            <div>
                <h4><img src="/static/icons/drive-upload.png" alt="" /> Re-upload</h4>
                <form action="/track/{{ track.tid }}/upload" method="post" enctype="multipart/form-data" data-upload>
                    <input type="file" name="qqfile" />
                    <input type="submit" value="Upload" />
                </form>
            </div>
        </div>

        <h4 class="edit-category">Description</h4>
        <div class="edit-container">
            <div class="edit-notes">
                <h4><img src="/static/icons/card-pencil.png" /> Notes</h4>
                <form action="/track/{{ track.tid }}/notes" method="post">
                    <textarea name="notes">{% if track.has_notes %}{{ track.notes }}{% endif %}</textarea><br />
                    <div class="legend">(tags: [b]old, [u]nderline, [i]talic)</div>
                    <input type="submit" value="Update description" />
                    <input name="nonce" type="hidden" value="{{ nonce }}"/>
                </form>
            </div>
        </div>

        <h4 class="edit-category">Misc</h4>
        <div class="edit-container">
            <div>
                <h4><img src="/static/icons/balloon-sound.png"> Broadcast</h4>
                <form action="/track/{{ track.tid }}/flags" method="post">
                    <input type="checkbox" name="airable" {% if track.airable | default(value=false) %}checked="checked"{% endif %} />
                    Celestia Radio<br />
                    <input type="submit" value="Update" />
                    <input name="nonce" type="hidden" value="{{ nonce }}"/>
                </form>
            </div>
            <div>
                <h4><img src="/static/icons/billboard.png" alt=""/> Feature</h4>
                <form action="/track/{{ track.tid }}/feature" method="post">
                    <input type="submit" value="Feature this on your profile"/>
                    <input name="nonce" type="hidden" value="{{ nonce }}"/>
                </form>
            </div>
            <div>
                <h4><img src="/static/icons/cross.png" alt="" style="margin-top: 0px;" /> Delete</h4>
                <form action="/track/{{ track.tid }}/delete" method="get">
                    <input type="submit" value="Delete track"/>
                </form>
            </div>
        </div>
        <div style="clear:both;"></div>
    </div>
    <script type="text/javascript">
        addListener(document, "DOMContentLoaded", function(){
            var edit = document.getElementById('track-edit');
            var title = document.getElementById('track-edit-title');
            edit.style.maxHeight = '27px';
            if(window.location.hash && edit.querySelector(window.location.hash))
                edit.style.maxHeight = '1500px';
        {% if track.is_hidden %}
            edit.style.maxHeight = '1500px';
        {% endif %}
            title.style.cursor = 'pointer';
            addListener(title, 'click', function() {
                edit.style.maxHeight = edit.style.maxHeight == '27px' ? '1500px' : '27px';
            });
        });
    </script>
    {% endif %}

    {{ m::eventlist(events=events) }}

    <form class="postcomment" action="/track/{{ track.tid }}/comment" method="post">
        {% if not logged_in | default(value=false) %}Name: <input type="text" name="name" /><br />{% endif %}
        <input class="honeypot" type="text" name="url" placeholder="If you can see this, don't fill it in." />
        <textarea name="msg"></textarea><br />
        <input type="submit" value="Post a comment" onclick="this.form.submit();this.disabled=true;return false;" />
        {% if logged_in | default(value=false) %}<input name="nonce" type="hidden" value="{{ nonce }}"/>{% endif %}
    </form>
</div>
