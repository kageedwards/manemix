{% import "html/macros.tpl" as m %}

<h2>{{ title }}</h2>

<div class="search">
    <form action="/users/search">
        <img src="/static/icons/magnifier.png" alt="Search" />
        <input type="text" name="q" />
        <input type="submit" value="Search" />
    </form>
</div>

{{ m::userlist(users=users) }}

{% if has_prev %}
<a href="?p={{ prev }}">&laquo; Previous page</a>
{% if has_next %} - {% endif %}
{% endif %}
{% if has_next %}
<a href="?p={{ next }}">Next page &raquo;</a>
{% endif %}
