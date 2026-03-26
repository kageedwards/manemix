{% import "html/macros.tpl" as m %}
<!DOCTYPE html>
<html>
    <head>
        <title>{% if found %}{{ track.title }} - {% endif %}Manehattan Mix</title>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
        <meta name="robots" content="noindex" />
        <link rel="stylesheet" type="text/css" href="/static/player.css" />
        <script type="text/javascript" src="/static/sm2.js"></script>
        <script type="text/javascript" src="/static/player.js"></script>
    </head>
    <body>
        <div id="player-embed">
            <div class="nohtml5-notice">Sorry, your browser does not support HTML5 audio. We recommend you upgrade to a modern browser, like <a href="http://mozilla.org/firefox/">Firefox</a> or <a href="http://chrome.google.com/">Chrome</a>.</div>

            {% if found %}
            <a href="/track/{{ track.tid }}" target="_blank" class="pic">
                {% if track.has_art %}<img src="/track/{{ track.tid }}/art/thumb" class="cover" alt="Cover" />{% endif %}
                {% if not track.has_art %}<img src="/static/logo.png" class="logo" alt="Manehattan Mix" />{% endif %}
            </a>
            <h3><a href="/track/{{ track.tid }}" target="_blank" title="{{ track.title }}">{{ track.title }}</a></h3>
            <h4>by <a href="/user/{{ track.uid }}" target="_blank" title="{{ track.username }}">{{ track.username }}</a></h4>
            {{ m::player(tid=track.tid, title=track.title, username=track.username, list=track.tid, count=1) }}
            {% endif %}

            {% if not found %}
            <a href="/" target="_blank" class="pic">
                <img src="/static/logo.png" class="logo" alt="Manehattan Mix" />
            </a>
            <h3 style="margin-bottom:10px;">Track not found</h3>
            {% endif %}

            <div style="clear:both;"></div>
        </div>
    </body>
</html>
