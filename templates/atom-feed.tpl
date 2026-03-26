<?xml version="1.0" encoding="UTF-8" ?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <id>{{ manemix_url }}{{ feed_url }}</id>
    <title>{{ what }} on Manemix</title>
    <updated>{{ updated }}</updated>
    {% for t in tracks %}
    <entry>
        <title>{{ t.title }}</title>
        <updated>{{ t.date }}</updated>
        <id>{{ manemix_url }}/track/{{ t.tid }}</id>
        <link rel="alternate" href="{{ manemix_url }}/track/{{ t.tid }}" />
        <link rel="enclosure" type="audio/ogg"  href="{{ manemix_url }}/track/{{ t.tid }}/vorbis" />
        <link rel="enclosure" type="audio/mpeg" href="{{ manemix_url }}/track/{{ t.tid }}/mp3" />
        <author>
            <name>{{ t.username }}</name>
            <uri>{{ manemix_url }}/user/{{ t.uid }}</uri>
        </author>
    </entry>
    {% endfor %}
</feed>
