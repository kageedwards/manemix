<form method="post" action="{{ delete_url }}">
    Do you really want to delete <b>{{ what }}</b> ?
    <input type="submit" value="Delete" name="confirm" />
    <input name="nonce" type="hidden" value="{{ nonce }}"/>
</form>
<a class="danger" href="{{ cancel_url }}">Cancel</a>
