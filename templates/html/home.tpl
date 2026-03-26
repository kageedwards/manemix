{% import "html/macros.tpl" as m %}

{% if latest_news is defined and latest_news %}
<div id="newsticker">
    <img src="/static/icons/newspaper.png" />
    <b>Latest news</b>: <a href="{{ latest_news.url }}">{{ latest_news.title }}</a>
</div>
{% endif %}
{% if news_items is defined and news_items | length > 1 %}
<script>
    var news = [
        {% for item in news_items %}
        { title: "{{ item.title }}", url: "{{ item.url }}" }{% if not loop.last %}, {% endif %}
        {% endfor %}
    ];
</script>
<script src="/static/ticker.js"></script>
{% endif %}

<div class="featurebox">
    <h3>
        <a href="/tracks/featured">Featured &raquo;</a>
    </h3>
    {{ m::tracklist(tracks=featured_tracks, list_id="featured") }}
</div>

<div class="leftcol">
    <h3>
        <a href="/tracks/latest">Latest</a>
        <a class="feed" title="Feed" href="/tracks/latest/atom"><span>(Feed)</span></a>
    </h3>
    {{ m::tracklist(tracks=latest_tracks, list_id="latest") }}
    <a class="more" href="/tracks/latest">More</a>
</div>

<div class="rightcol">
    <h3><a href="/tracks/random">Random</a></h3>
    {{ m::tracklist(tracks=random_tracks, list_id="random") }}
    <a class="more" href="/tracks/random">More</a>
</div>

<div style="clear:both;"></div>
