package com.gemwallet.android.features.onboarding.presents.terms

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.onboarding.viewmodels.terms.acceptTermRows
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.SelectionIndicator
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.PublicUrl

@Composable
fun AcceptTermsScreen(onCancel: CancelAction, onAccept: () -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val items = remember { acceptTermRows() }
    val accepted = remember { mutableStateMapOf<Int, Boolean>() }
    Scene(
        title = stringResource(R.string.onboarding_accept_terms_title),
        onClose = { onCancel() },
        mainAction = {
            MainActionButton(
                title = stringResource(R.string.onboarding_accept_terms_continue),
                state = buttonState(enabled = items.indices.all { accepted[it] == true }),
                onClick = { onAccept() },
            )
        },
        actions = {
            IconButton(
                {
                    uriHandler.open(context, AppUrl.page(PublicUrl.TERMS_OF_SERVICE))
                },
            ) {
                Icon(AppIcons.InfoOutlined, "")
            }
        },
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = sceneContentPadding()),
        ) {
            item {
                Text(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = sceneContentPadding()),
                    text = stringResource(R.string.onboarding_accept_terms_message),
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.secondary,
                )
                Spacer(Modifier.size(paddingDefault))
            }
            items.forEachIndexed { index, item ->
                termItem(
                    isUnderstand = accepted[index] == true,
                    description = item.description,
                    testTag = "term_${index + 1}",
                ) { accepted[index] = accepted[index] != true }
            }

            item { Spacer(modifier = Modifier.size(it.calculateBottomPadding())) }
        }
    }
}

private fun LazyListScope.termItem(isUnderstand: Boolean, @StringRes description: Int, testTag: String, onClick: () -> Unit) {
    item {
        Card(
            modifier = Modifier
                .testTag(testTag)
                .clip(shape = RoundedCornerShape(paddingDefault))
                .clickable(onClick = onClick),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
            elevation = CardDefaults.cardElevation(paddingHalfSmall),
            shape = RoundedCornerShape(paddingDefault),
        ) {
            Row(modifier = Modifier.defaultPadding(), verticalAlignment = Alignment.CenterVertically) {
                SelectionIndicator(isSelected = isUnderstand)
                Spacer16()
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = stringResource(description),
                        style = MaterialTheme.typography.bodyLarge,
                        color = if (isUnderstand) {
                            MaterialTheme.colorScheme.onSurface
                        } else {
                            MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f)
                        },
                    )
                }
            }
        }
        Spacer(Modifier.size(paddingDefault))
    }
}

@Preview
@Composable
fun AcceptTermsScreenPreview() {
    WalletTheme {
        AcceptTermsScreen({}) { }
    }
}
