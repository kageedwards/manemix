{% import "html/macros.tpl" as m %}

<div class="playlist">

    {% if firstrun | default(value=false) %}
    <div class="message">Playlist created.<br/> To add a track to it, navigate to the track's page and look for an <img src="/static/icons/playlist-add.png" alt=""/> icon next to the title.</div>
    {% endif %}

    <h2>{{ playlist.playlist_name }}</h2>
    <h4>by <a href="/user/{{ playlist.uid }}">{{ playlist.username }}</a></h4>

    {% if playlist.has_description %}
    <div class="notes">{{ playlist.description_html | safe }}</div>
    {% endif %}

    <a name="tracks"></a>
    {{ m::tracklist(tracks=tracks, list_id="playlist") }}

    {% if is_owner | default(value=false) %}
    <div class="dialog">
        <h3><img src="/static/icons/pencil.png" /> Edit</h3>
        <form method="post" action="/playlist/{{ playlist.playlist_id }}/edit">
            <table>
                <tr>
                    <td><img src="/static/icons/rename.png"> Name:</td>
                    <td><input type="text" name="name" value="{{ playlist.playlist_name }}" /></td>
                </tr>
                <tr>
                    <td>
                        <img src="/static/icons/card-pencil.png"> Notes:<br />
                        <span class="legend">(tags: [b]old, [u]nderline, [i]talic)</span>
                    </td>
                    <td><textarea name="desc">{% if playlist.has_description %}{{ playlist.description }}{% endif %}</textarea></td>
                </tr>
                <tr>
                    <td></td>
                    <td><input type="submit" value="Update" /></td>
                </tr>
            </table>
            <input name="nonce" type="hidden" value="{{ nonce }}"/>
        </form>
        <form action="/playlist/{{ playlist.playlist_id }}/feature" method="post">
            <img src="/static/icons/billboard.png" alt=""/>
            <input type="submit" value="Feature this on your profile"/>
            <input name="nonce" type="hidden" value="{{ nonce }}"/>
        </form>
        <form action="/playlist/{{ playlist.playlist_id }}/delete">
            <img src="/static/icons/cross.png" alt=""/>
            <input type="submit" value="Delete playlist"/>
        </form>
        <div style="clear:both;"></div>
    </div>
    {% endif %}

</div>
