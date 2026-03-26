<h2>Your account</h2>

{% if has_message | default(value=false) %}<div class="message">{{ message }}</div>{% endif %}
{% if has_error | default(value=false) %}<div class="error">{{ error }}</div>{% endif %}

<form action="/account" method="post">
    <table>
        <tr>
            <td><label for="r_name">Display name:</label>
            <br/><span class="legend">(will show up in artist tag for your tracks)</span>
            </td>
            <td><input id="r_name" name="name" value="{{ account.username }}" maxlength="200" /></td>
        </tr>
        <tr>
            <td><label for="r_email">Email:</label>
            <br/><span class="legend">(publicly visible)</span>
            </td>
            <td><input id="r_email" name="email" value="{{ account.email }}" /></td>
        </tr>
        <tr>
            <td><label for="r_notify">Email notifications:</label>
            <br/><span class="legend">(for followed artists' releases, comments)</span>
            </td>
            <td><input id="r_notify" name="notify" type="checkbox" {% if account.notify | default(value=false) %}checked="checked"{% endif %} /></td>
        </tr>
        <tr>
            <td><label for="r_about">Description:</label><br />
            <span class="legend">(tags: [b]old, [u]nderline, [i]talic)</span></td>
            <td><textarea id="r_about" name="about">{% if account.has_about %}{{ account.about }}{% endif %}</textarea></td>
        </tr>
        <tr>
            <td><label for="r_license">Default license:</label></td>
            <td><b>{{ account.license }}</b> <a href="/account/license">(change)</a></td>
        </tr>
        <tr>
            <td><label for="r_theme">Theme:</label></td>
            <td>
                <select id="r_theme" name="theme">
                    <option value="auto" {% if account.theme == "auto" %}selected{% endif %}>Auto (follow system)</option>
                    <option value="light" {% if account.theme == "light" %}selected{% endif %}>Light</option>
                    <option value="dark" {% if account.theme == "dark" %}selected{% endif %}>Dark</option>
                </select>
            </td>
        </tr>
        <tr>
            <td><label for="r_oldpw">Old password:</label></td>
            <td><input type="password" id="r_oldpw" name="oldpw" value="{{ old_password | default(value='') }}" /></td>
        </tr>
        <tr>
            <td><label for="r_newpw">New password:</label></td>
            <td><input type="password" id="r_newpw" name="newpw" /></td>
        </tr>
        <tr>
            <td><label for="r_newpwconf">Confirm new password:</label></td>
            <td><input type="password" id="r_newpwconf" name="newpwconf" /></td>
        </tr>
    </table>
    <input name="nonce" type="hidden" value="{{ nonce }}"/>
    <input type="submit" value="Update" />
</form>

<h4><img src="/static/icons/cross.png" alt="" style="margin-top: 0px;" /> Delete</h4>
<form action="/account/delete" method="get">
    <input type="submit" value="Delete your account"/>
</form>
