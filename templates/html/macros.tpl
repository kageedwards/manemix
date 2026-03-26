{% macro tracklist(tracks, list_id) %}
<ul class="tracklist">
    {% if tracks | length == 0 %}
    <li class="empty">Nothing here.</li>
    {% endif %}
    {% for t in tracks %}
    <li {% if t.is_hidden %}class="hidden"{% endif %} onclick="toggle(this.querySelector('.player'));">
        {% if t.has_art %}<img class="cover" alt="" src="/track/{{ t.tid }}/art/thumb" />{% endif %}
        <div class="title"><a href="/track/{{ t.tid }}" title="{{ t.title }}">{{ t.title }}</a></div>
        <div class="by">by <a href="/user/{{ t.uid }}" title="{{ t.username }}">{{ t.username }}</a></div>
        <div style="clear:both;"></div>
        {{ self::player(tid=t.tid, title=t.title, username=t.username, list=list_id, count=loop.index) }}
    </li>
    {% endfor %}
</ul>
{% endmacro tracklist %}

{% macro player(tid, title, username, list, count) %}
<div id="{{ list }}-{{ count }}" class="player"></div>
<script>
    registerTrack({
        id: "{{ list }}-{{ count }}",
        tid: "{{ tid }}",
        title: "{{ title }}",
        artist: "{{ username }}",
        mp3: "/track/{{ tid }}/mp3?stream=1",
        vorbis: "/track/{{ tid }}/vorbis?stream=1",
        aac: "/track/{{ tid }}/aac?stream=1",
        opus: "/track/{{ tid }}/opus?stream=1",
        list: "{{ list }}"
    });
</script>
{% endmacro player %}

{% macro player_fallback(tid) %}
<noscript>
    <audio controls="">
        <source type="audio/ogg; codecs=opus" src="/track/{{ tid }}/opus?stream=1" />
        <source type="audio/aac" src="/track/{{ tid }}/aac?stream=1" />
        <source type="audio/ogg" src="/track/{{ tid }}/vorbis?stream=1" />
        <source type="audio/mpeg" src="/track/{{ tid }}/mp3?stream=1" />
    </audio>
</noscript>
{% endmacro player_fallback %}

{% macro avatar(email_md5, is_self) %}
<a class="avatar" {% if is_self %}href="http://gravatar.com/emails/" target="_blank"{% endif %}>
    <img src="https://secure.gravatar.com/avatar/{{ email_md5 }}?d=https%3A%2F%2Fmanemix.org%2Fstatic%2Favatar.png" alt="" />
</a>
{% endmacro avatar %}

{% macro embed_code(manemix_url, tid, title, uid, username, width) %}
<iframe width="{{ width }}px" height="150px" frameborder="0" src="{{ manemix_url }}/track/{{ tid }}/embed"><a href="{{ manemix_url }}/track/{{ tid }}">{{ title }}</a> by <a href="{{ manemix_url }}/user/{{ uid }}">{{ username }}</a></iframe>
{% endmacro embed_code %}

{% macro userlist(users) %}
<ul class="userlist">
    {% if users | length == 0 %}
    <li class="empty">Nobody here.</li>
    {% endif %}
    {% for u in users %}
    <li>
        {{ self::avatar(email_md5=u.email_md5, is_self=u.is_self) }}
        <a class="name" href="/user/{{ u.uid }}">{{ u.username }}</a>
        {% if u.has_about %}<div class="about">{{ u.about_html | safe }}</div>{% endif %}
        <div style="clear:both;"></div>
    </li>
    {% endfor %}
</ul>
{% endmacro userlist %}

{% macro eventlist(events) %}
<div class="events_wrapper">
<ul class="events">
    <h4><img src="/static/icons/fire-small.png" alt="" /> Recent happenings</h4>
    {% for e in events %}
    <li class="event" name="{{ e.event_id }}">
        <span class="date" title="{{ e.utc_date }} UTC">{{ e.fuzzy_time }}</span>

        {% if e.is_publish %}
        <img src="/static/icons/disc-arrow.png" alt="" />
        <a href="/user/{{ e.source_uid }}">{{ e.source_name }}</a>
        published {% if e.has_track %}<a href="/track/{{ e.tid }}">{{ e.track_title }}</a>{% endif %}.
        {% endif %}

        {% if e.is_comment %}
        <img src="/static/icons/balloon-white-left.png" alt="" />
        <a href="/user/{{ e.source_uid }}">{{ e.source_name }}</a>
        {% if e.has_track %}posted a comment{% else %}said{% endif %}
        on
        {% if e.has_track %}<a href="/track/{{ e.tid }}">{{ e.track_title }}</a> by <a href="/user/{{ e.target_uid }}">{{ e.target_name }}</a>:{% endif %}
        {% if not e.has_track %}<a href="/user/{{ e.target_uid }}">{{ e.target_name }}'s profile</a>:{% endif %}
        <p class="comment">{{ e.message_html | safe }}</p>
        {% endif %}

        {% if e.is_follow %}
        <img src="/static/icons/plus.png" alt="" />
        <a href="/user/{{ e.source_uid }}">{{ e.source_name }}</a>
        started following <a href="/user/{{ e.target_uid }}">{{ e.target_name }}</a>.
        {% endif %}

        {% if e.is_favorite %}
        <img src="/static/icons/star.png" alt="" />
        <a href="/user/{{ e.source_uid }}">{{ e.source_name }}</a>
        favorited {% if e.has_track %}<a href="/track/{{ e.tid }}">{{ e.track_title }}</a>{% endif %}
        by <a href="/user/{{ e.target_uid }}">{{ e.target_name }}</a>.
        {% endif %}
    </li>
    {% endfor %}
    {% if events | length == 0 %}
    <li class="empty">Nothing here.</li>
    {% endif %}
</ul>
</div>
{% endmacro eventlist %}

{% macro license_badge(track) %}
<div class="license">
    {% if track.is_copyright %}
    Copyright &copy; {{ track.username }}
    {% elif track.license_key == "cc_by" %}
    License: <a href="http://creativecommons.org/licenses/by/3.0/">CC BY</a>
    {% elif track.license_key == "cc_by_nc" %}
    License: <a href="http://creativecommons.org/licenses/by-nc/3.0/">CC BY-NC</a>
    {% elif track.license_key == "cc_by_sa" %}
    License: <a href="http://creativecommons.org/licenses/by-sa/3.0/">CC BY-SA</a>
    {% elif track.license_key == "cc_by_nd" %}
    License: <a href="http://creativecommons.org/licenses/by-nd/3.0/">CC BY-ND</a>
    {% elif track.license_key == "cc_by_nc_sa" %}
    License: <a href="http://creativecommons.org/licenses/by-nc-sa/3.0/">CC BY-NC-SA</a>
    {% elif track.license_key == "cc_by_nc_nd" %}
    License: <a href="http://creativecommons.org/licenses/by-nc-nd/3.0/">CC BY-NC-ND</a>
    {% elif track.license_key == "public" %}
    Public domain <a href="https://creativecommons.org/publicdomain/zero/1.0/">(CC0)</a>
    {% else %}
    License: {{ track.license }}
    {% endif %}
</div>
{% endmacro license_badge %}
