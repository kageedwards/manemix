<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<oembed>
    <version>1.0</version>
    <provider_name>Manehattan Mix</provider_name>
    <provider_url>{{ manemix_url }}</provider_url>
    <type>video</type>
    <url>{{ manemix_url }}/track/{{ track.tid }}</url>
    <width>{{ width }}</width>
    <height>150</height>
    <html>{{ embed_html }}</html>
    <title>{{ track.title }}</title>
    <author_name>{{ track.username }}</author_name>
    <author_url>{{ manemix_url }}/user/{{ track.uid }}</author_url>
</oembed>
