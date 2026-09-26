package com.gemwallet.android.features.rewards.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.emoji
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import uniffi.gemstone.GemRewardsIntroItem
import uniffi.gemstone.GemRewardsInviteAction

internal fun LazyListScope.rewardsHead(description: String, intro: List<GemRewardsIntroItem>, action: GemRewardsInviteAction?, onGetStarted: () -> Unit, onShare: () -> Unit) {
    item {
        Column(
            modifier = Modifier
                .listItem(ListPosition.Single)
                .padding(paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(paddingDefault),
        ) {
            Spacer16()
            Text("\uD83C\uDF81", fontSize = 64.sp)
            Text(
                text = parseMarkdownToAnnotatedString(
                    markdown = stringResource(R.string.rewards_invite_friends_title),
                ),
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.headlineSmall,
            )
            Text(
                text = parseMarkdownToAnnotatedString(markdown = description),
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(paddingDefault),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                intro.forEach { item ->
                    Column(
                        modifier = Modifier.weight(1f),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
                    ) {
                        Text(
                            text = item.emoji(),
                            fontSize = 26.sp,
                        )
                        Text(
                            text = stringResource(item.titleRes()),
                            color = MaterialTheme.colorScheme.secondary,
                            style = MaterialTheme.typography.bodyMedium,
                            textAlign = TextAlign.Center,
                        )
                    }
                }
            }
            when (action) {
                GemRewardsInviteAction.CREATE_CODE -> MainActionButton(
                    title = stringResource(R.string.common_get_started),
                    onClick = onGetStarted,
                )

                GemRewardsInviteAction.SHARE -> MainActionButton(
                    onClick = onShare,
                ) {
                    Icon(AppIcons.Share, contentDescription = "share")
                    Spacer8()
                    Text(stringResource(R.string.rewards_invite_friends_title))
                }

                null -> Unit
            }
        }
    }
}
