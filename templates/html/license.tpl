{% if has_track | default(value=false) %}
<h2>License for <a href="{{ track_url }}">{{ track_title }}</a></h2>
{% else %}
<h2>Global license</h2>
{% endif %}

<form method="post">
<table class="licenses">
    <tr>
        <td><input name="license" type="radio" value="Copyright" id="lc" {% if current_license == "Copyright" %}checked="checked"{% endif %} /></td>
        <td>
            <b><label for="lc">Copyright</label></b>
            <div>Default license. Most restrictive.</div>
        </td>
    </tr>
    <tr>
        <td><input name="license" type="radio" value="CC BY-NC" id="lnc" {% if current_license == "CC BY-NC" %}checked="checked"{% endif %} /></td>
        <td>
            <b><label for="lnc">Creative Commons: Attribution-NonCommercial (CC BY-NC)</label></b>
            <div>
                &ldquo;This license lets others remix, tweak, and build upon your work non-commercially, and although their new works must also acknowledge you and be non-commercial, they don't have to license their derivative works on the same terms.&rdquo; <a href="http://creativecommons.org/licenses/by-nc/3.0">(full license)</a><br />
                <a href="http://creativecommons.org/licenses/">More about Creative Commons licenses.</a><br /><br />
                Other CC licenses:
                <input name="license" type="radio" value="CC BY" id="lby" {% if current_license == "CC BY" %}checked="checked"{% endif %} /> <label for="lby">Attribution <a href="http://creativecommons.org/licenses/by/3.0">(BY)</a></label>
                <input name="license" type="radio" value="CC BY-SA" id="lsa" {% if current_license == "CC BY-SA" %}checked="checked"{% endif %} /> <label for="lsa">Attribution-ShareAlike <a href="http://creativecommons.org/licenses/by-sa/3.0">(BY-SA)</a></label>
                <input name="license" type="radio" value="CC BY-ND" id="lnd" {% if current_license == "CC BY-ND" %}checked="checked"{% endif %} /> <label for="lnd">Attribution-NoDerivs <a href="http://creativecommons.org/licenses/by-nd/3.0">(BY-ND)</a></label>
                <input name="license" type="radio" value="CC BY-NC-SA" id="lncsa" {% if current_license == "CC BY-NC-SA" %}checked="checked"{% endif %} /> <label for="lncsa">Attribution-NonCommercial-ShareAlike <a href="http://creativecommons.org/licenses/by-nc-sa/3.0">(BY-NC-SA)</a></label>
                <input name="license" type="radio" value="CC BY-NC-ND" id="lncnd" {% if current_license == "CC BY-NC-ND" %}checked="checked"{% endif %} /> <label for="lncnd">Attribution-NonCommercial-NoDerivs <a href="http://creativecommons.org/licenses/by-nc-nd/3.0">(BY-NC-ND)</a></label>
            </div>
        </td>
    </tr>
    <tr>
        <td><input name="license" type="radio" value="Public domain" id="lpub" {% if current_license == "Public domain" %}checked="checked"{% endif %} /></td>
        <td>
            <b><label for="lpub">Public Domain <a href="https://creativecommons.org/publicdomain/zero/1.0/">(CC0)</a></label></b>
            <div>"No rights reserved". Use it if you make your music for fun and want everyone to make the most of it.</div>
        </td>
    </tr>
    <tr>
        <td><input name="license" type="radio" value="custom" id="lcus" /></td>
        <td>
            <b><label for="lcus">Custom:</label></b>
            <input name="custom-license" value=""/>
        </td>
    </tr>
    {% if has_track %}
    <tr>
        <td><input name="mkdefault" type="checkbox" id="ldef" /></td>
        <td><label for="ldef">Make it the default license for new tracks.</label></td>
    </tr>
    {% endif %}
    <tr>
        <td><input name="retro" type="checkbox" id="lall"/></td>
        <td><label for="lall">Apply to all tracks.</label></td>
    </tr>
    <tr>
        <td colspan="2"><input type="submit" value="Update" /></td>
    </tr>
</table>
<input name="nonce" type="hidden" value="{{ nonce }}"/>
</form>
