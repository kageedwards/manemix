{% import "html/macros.tpl" as m %}

<h2>
    {{ title }}
    {% if feed_url is defined and feed_url %}<a class="feed" title="Feed" href="{{ feed_url }}"><span>(Feed)</span></a>{% endif %}
</h2>

{% if search is defined and search %}
<div class="search">
    <form action="/tracks/search">
        <img src="/static/icons/magnifier.png" alt="Search" />
        <input type="text" name="q" value="{{ search }}" />
        <input type="submit" value="Search" />
        <span class="legend">(Advanced search tags: <kbd>title:</kbd> <kbd>artist:</kbd> <kbd>license:</kbd>)</span>
    </form>
</div>
{% endif %}

{{ m::tracklist(tracks=tracks, list_id="tracklist") }}

{% if has_prev %}
<a href="?p={{ prev }}">&laquo; Newer tracks</a>
{% if has_next %} - {% endif %}
{% endif %}
{% if has_next %}
<a href="?p={{ next }}">Older tracks &raquo;</a>
{% endif %}
