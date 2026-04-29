package com.gemwallet.android.features.onboarding

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Circle
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.PublicUrl

@Composable
fun AcceptTermsScreen(
    onCancel: CancelAction,
    onAccept: () -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    var isUnderstand1 by remember { mutableStateOf(false) }
    var isUnderstand2 by remember { mutableStateOf(false) }
    var isUnderstand3 by remember { mutableStateOf(false) }
    Scene(
        title = stringResource(R.string.onboarding_accept_terms_title),
        onClose = { onCancel() },
        mainAction = {
            MainActionButton(
                title = stringResource(R.string.onboarding_accept_terms_continue),
                enabled = isUnderstand1 && isUnderstand2 && isUnderstand3,
                onClick = { onAccept() }
            )
        },
        actions = {
            IconButton(
                {
                    uriHandler.open(context, AppUrl.page(PublicUrl.TERMS_OF_SERVICE))
                }
            ) {
                Icon(Icons.Outlined.Info, "")
            }
        },
    ) {
        LazyColumn (
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
            termItem(
                isUnderstand1,
                R.string.onboarding_accept_terms_item1_message,
                testTag = "term_1",
            ) { isUnderstand1 = !isUnderstand1 }
            termItem(
                isUnderstand2,
                R.string.onboarding_accept_terms_item2_message,
                testTag = "term_2",
            ) { isUnderstand2 = !isUnderstand2 }
            termItem(
                isUnderstand3,
                R.string.onboarding_accept_terms_item3_message,
                testTag = "term_3",
            ) { isUnderstand3 = !isUnderstand3 }

            item { Spacer(modifier = Modifier.size(it.calculateBottomPadding())) }
        }
    }
}

private fun LazyListScope.termItem(
    isUnderstand: Boolean,
    @StringRes description: Int,
    testTag: String,
    onClick: () -> Unit,
) {
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
                Box(
                    modifier = Modifier.size(compactIconSize),
                    contentAlignment = Alignment.Center,
                ) {
                    if (isUnderstand) {
                        SelectionCheckmark()
                    } else {
                        Icon(
                            modifier = Modifier.size(compactIconSize),
                            imageVector = Icons.Outlined.Circle,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.secondary,
                        )
                    }
                }
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
