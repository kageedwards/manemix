<!DOCTYPE html>
<html>
    <head>
        <title>{% if has_title %}{{ title }} - {% endif %}Manehattan Mix</title>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
        <link rel="stylesheet" type="text/css" href="/static/style.css" />
        <link rel="stylesheet" type="text/css" href="/static/dark.css" />
        <link rel="shortcut icon" href="/static/favicon.ico" id="favicon-link" />
        <script type="text/javascript" src="/static/player.js"></script>
        <script type="text/javascript" src="/static/upload.js"></script>
        {% if requires_stats_js | default(value=false) %}
        <script type="text/javascript" src="/static/d3.js"></script>
        <script type="text/javascript" src="/static/stats.js"></script>
        {% endif %}
        {% if has_oembed | default(value=false) %}
        <link rel="alternate" type="application/json+oembed" href="http://manemix.org/oembed?format=json&amp;url={{ manemix_url | urlencode }}/track/{{ tid }}">
        <link rel="alternate" type="application/xml+oembed" href="http://manemix.org/oembed?format=xml&amp;url={{ manemix_url | urlencode }}/track/{{ tid }}">
        {% endif %}
        {% if feed_url is defined and feed_url %}<link href="{{ feed_url }}" type="application/atom+xml" rel="alternate" />{% endif %}
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no" />
        <script>
        (function(){
            var t = '{{ theme | default(value="auto") }}';
            if (t === 'auto') {
                t = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
            }
            document.documentElement.setAttribute('data-theme', t);
        })();
        </script>
    </head>
    <body>
        <div id="main">
            <div class="nohtml5-notice"><b>Notice</b>: Manemix requires a browser with HTML5 audio support, which your browser does not seem to have. You will not be able to play tracks with your current setup. We recommend you upgrade to a modern browser, like <a href="http://mozilla.org/firefox/">Firefox</a> or <a href="http://chrome.google.com/">Chrome</a>.</div>
            <div id="header">
                <h1><a href="/">Manehattan Mix</a></h1>
                <div id="logstatus">
                    {% if logged_in %}
                    Hi <b><a href="/user/{{ session_uid }}">{{ session_username }}</a></b>.
                    <a href="/logout">Logout</a>
                    {% else %}
                    <a href="/login">Login</a>
                    {% endif %}
                    <button id="theme-toggle" type="button" aria-label="Toggle theme" onclick="toggleTheme()">🌓</button>
                </div>
                <div id="navbar">
                    <a href="/">Home</a>
                    <a href="/tracks/latest">Latest</a>
                    <a href="/artists">Artists</a>
                    <form action="/tracks/search">
                        <button>
                            <img alt="Search" src="/static/icons/purple-magnifier.png"/>
                        </button>
                        <input type="text" name="q" placeholder="Search…" value="{{ search | default(value='') }}" />
                    </form>
                </div>
                <div style="clear:both;"></div>
            </div>
            <div id="contents">
                {{ body | safe }}
            </div>
            <div id="footer">
                <a href="/faq">FAQ</a>
                <a href="/api">API</a>
                <a href="https://github.com/manemix/manemix/">Code</a>
                -
                <a href="http://blog.manemix.org/">Blog</a>
                <a title="#manemix on irc.ponychat.net" href="http://webchat.ponychat.net/?autojoin=%23manemix">IRC</a>
                <a href="/thanks">Thanks</a>
                {# .br. contact@manemix.org #}
            </div>
        </div>
        <script type="text/javascript">document.body.className = 'js';</script>
        <script>
        function toggleTheme() {
            var html = document.documentElement;
            var current = html.getAttribute('data-theme');
            var next = (current === 'dark') ? 'light' : 'dark';
            html.setAttribute('data-theme', next);
            document.cookie = 'theme=' + next + ';path=/;max-age=31536000;SameSite=Lax';
            {% if logged_in %}
            fetch('/account/theme', {
                method: 'POST',
                headers: {'Content-Type': 'application/x-www-form-urlencoded'},
                body: 'theme=' + next + '&nonce={{ nonce | default(value="") }}'
            });
            {% endif %}
        }
        // Listen for OS theme changes when preference is 'auto'
        (function(){
            var pref = '{{ theme | default(value="auto") }}';
            if (pref === 'auto') {
                window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
                    document.documentElement.setAttribute('data-theme', e.matches ? 'dark' : 'light');
                });
            }
        })();
        </script>
    </body>
</html>
