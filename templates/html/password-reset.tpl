<h2>Reset your password</h2>

{% if has_message | default(value=false) %}<div class="message">{{ message }}</div>{% endif %}
{% if has_error | default(value=false) %}<div class="error">{{ error }}</div>{% endif %}

<form action="/account/reset" method="post">
    Email: <input name="email" /> <input type="submit" value="Reset" />
</form>
