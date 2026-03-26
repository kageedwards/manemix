{% import "html/macros.tpl" as m %}

<div class="user">

{% if welcome | default(value=false) %}
<div class="message">
All set! You can now start following artists and making playlists. If you are an artist yourself you may want to <a href="#track-uploader">upload tracks</a> below and edit your <a href="/account">profile</a>.
</div>
{% endif %}

{{ m::avatar(email_md5=account.email_md5, is_self=is_self) }}

<h2>
    {{ account.username }}
    {% if logged_in | default(value=false) and not is_self %}
        {% if is_followed | default(value=false) %}
        <form action="/user/{{ account.uid }}/unfollow" method="post" style="display:inline;">
            <input name="nonce" type="hidden" value="{{ nonce }}" />
            <button type="submit" class="follow"><span>Stop following</span></button>
        </form>
        {% else %}
        <form action="/user/{{ account.uid }}/follow" method="post" style="display:inline;">
            <input name="nonce" type="hidden" value="{{ nonce }}" />
            <button type="submit" class="follow disabled"><span>Follow</span></button>
        </form>
        {% endif %}
    {% elif not logged_in | default(value=false) and not is_self %}
        <a class="follow disabled" href="/login?redirect=/user/{{ account.uid }}"><span>Follow</span></a>
    {% endif %}
</h2>

<div class="items">
{% if is_self %}<div class="item"><img src="/static/icons/card-pencil.png" alt="" /> <a href="/account">Edit</a></div>{% endif %}
{% if account.has_favs %}<div class="item"><img src="/static/icons/star.png" alt="" /> <a href="/user/{{ account.uid }}/favorites">Favorite tracks ({{ account.num_favs }})</a></div>{% endif %}
{% if account.has_followers %}<div class="item"><img src="/static/icons/users.png" alt="" /> {{ account.num_followers }} follower{% if account.followers_plural %}s{% endif %}</div>{% endif %}
</div>
<div style="clear:both;"></div>

{% if feature.has_featured %}
<div class="featurebox">
    {% if is_self %}
    <form class="defeature" action="/user/{{ account.uid }}/defeature" method="post">
        <button class="fakelink" type="submit"><span>Remove featured content</span> &#10006;</button>
        <input name="nonce" type="hidden" value="{{ nonce | default(value='') }}"/>
    </form>
    {% endif %}
    {% if feature.featured_title %}
    <h3>
        <a href="{{ feature.featured_link }}">{{ feature.featured_title }} &raquo;</a>
    </h3>
    {% endif %}
    {{ m::tracklist(tracks=feature.tracks, list_id="featured") }}
</div>
{% endif %}

{% if account.has_about %}<div class="notes">{{ account.about_html | safe }}</div>{% endif %}

</div>

{{ m::eventlist(events=events) }}

<h3><img src="/static/icons/disc.png" alt="" /> Tracks</h3>
{{ m::tracklist(tracks=tracks, list_id="tracks") }}
{% if is_self %}
<div id="track-uploader">
    <form action="/track/new" method="post" enctype="multipart/form-data" data-upload>
        <input type="file" name="qqfile" />
        <input type="submit" value="Upload new track" />
    </form>
</div>
{% endif %}

{% if is_self %}
<form class="newplaylist" action="/playlist/new" method="post">
    <input type="text" name="name" placeholder="Name" />
    <input type="submit" value="New playlist" />
    <input name="nonce" type="hidden" value="{{ nonce | default(value='') }}"/>
</form>
{% endif %}
<h3>Playlists</h3>
{% if not has_playlists %}<div class="playlists empty">Nothing here.</div>{% endif %}
{% if has_playlists %}
<ul class="playlists">
{% for p in playlists %}
    <li>
        <div class="title">
            <a href="/playlist/{{ p.playlist_id }}">{{ p.playlist_name }}</a>
            <span class="count">{{ p.track_count }}</span>
        </div>
        {% if p.has_description %}
        <div class="description">{{ p.description_html | safe }}</div>
        {% endif %}
    </li>
{% endfor %}
</ul>
{% endif %}
