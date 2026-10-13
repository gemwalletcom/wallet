package com.gemwallet.android.features.rewards.presents.dialogs

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.filters.FormDialog
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer16

@Composable
internal fun RedeemRewardsCodeDialog(isVisible: Boolean, referralCode: String, isLoading: Boolean, onCode: (String) -> Unit, onDismiss: () -> Unit) {
    var code by remember(isVisible, referralCode) { mutableStateOf(referralCode) }
    val context = LocalContext.current

    val focusRequester = remember { FocusRequester() }

    val doneAction: () -> Unit = done@{
        if (code.isEmpty() || isLoading) return@done
        context.requestAuth(AuthRequest.Default) { onCode(code) }
    }
    val done: @Composable () -> Unit = {
        TextButton(
            onClick = doneAction,
            enabled = code.isNotEmpty() && !isLoading,
        ) {
            if (isLoading) {
                CircularProgressIndicator16()
            } else {
                Text(stringResource(R.string.common_done))
            }
        }
    }
    FormDialog(
        isVisible = isVisible,
        title = stringResource(R.string.rewards_referral_code),
        onDismiss = onDismiss,
        doneAction = done,
    ) {
        LaunchedEffect(Unit) {
            focusRequester.requestFocus()
        }
        GemTextField(
            modifier = Modifier
                .focusRequester(focusRequester)
                .fillMaxWidth(),
            label = stringResource(id = R.string.rewards_referral_code),
            value = code,
            onValueChange = {
                code = it
            },
            keyboardActions = KeyboardActions(onDone = { doneAction() }),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            singleLine = true,
        )
        Spacer16()
    }
}
