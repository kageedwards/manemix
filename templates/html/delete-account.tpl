<form method="post">
    Do you really want to delete your account?<br />
    Every information related to your account and your tracks are going to be permanently erased. This button is the last step, and is irreversible. <br />
    <input type="submit" value="Delete" name="confirm" />
    <input name="nonce" type="hidden" value="{{ nonce }}"/>
</form>
<a class="danger" href="{{ cancel_url }}">Cancel</a>
