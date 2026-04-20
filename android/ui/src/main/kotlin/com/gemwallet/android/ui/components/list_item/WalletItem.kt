package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType

@Composable
fun WalletItem(
    wallet: Wallet,
    isCurrent: Boolean,
    modifier: Modifier = Modifier,
    listPosition: ListPosition,
    onEdit: ((String) -> Unit)? = null,
) {
    WalletItem(
        modifier = modifier,
        id = wallet.id,
        name = wallet.name,
        walletAddress = wallet.accounts.firstOrNull()?.address ?: "",
        walletChain = wallet.accounts.firstOrNull()?.chain,
        isCurrent = isCurrent,
        type = wallet.type,
        listPosition = listPosition,
        onEdit = onEdit
    )
}

@Composable
fun WalletItem(
    id: String,
    name: String,
    walletAddress: String?,
    walletChain: Chain?,
    isCurrent: Boolean,
    type: WalletType,
    modifier: Modifier = Modifier,
    listPosition: ListPosition,
    onEdit: ((String) -> Unit)? = null,
) {
    ListItem(
        modifier = modifier,
        minHeight = ListItemDefaults.iconMinHeight,
        titleSubtitleSpacing = space0,
        trailingContentEndPadding = paddingSmall,
        leading = @Composable {
            IconWithBadge(
                icon = walletItemIconModel(type = type, walletChain = walletChain),
                supportIcon = type.supportIcon(),
            )
        },
        title = {
            ListItemTitleText(text = name)
        },
        subtitle = {
            val subtitle = when (type) {
                WalletType.Multicoin -> stringResource(R.string.wallet_multicoin)
                else -> walletAddress?.let {
                    AddressFormatter(it, chain = walletChain, style = AddressFormatter.Style.Extra(1)).value()
                } ?: ""
            }
            ListItemSupportText(subtitle)
        },
        listPosition = listPosition,
        trailing = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Spacer16()
                if (isCurrent) {
                    SelectionCheckmark()
                }
                if (onEdit != null) {
                    Spacer8()
                    WalletEditButton(onClick = { onEdit(id) })
                }
            }
        }
    )
}

@Composable
private fun WalletEditButton(
    onClick: () -> Unit,
) {
    IconButton(onClick = onClick) {
        Icon(
            imageVector = Icons.Outlined.Settings,
            contentDescription = "edit",
            tint = MaterialTheme.colorScheme.secondary,
        )
    }
}

fun walletItemIconModel(type: WalletType, walletChain: Chain?): Any? = when (type) {
    WalletType.Multicoin -> R.drawable.multicoin_wallet
    WalletType.Single,
    WalletType.PrivateKey,
    WalletType.View -> walletChain?.getIconUrl()
}

@Preview
@Composable
fun PreviewWalletItem() {
    MaterialTheme {
        WalletItem(
            id = "1",
            name = "Foo wallet name",
            walletChain = Chain.Ethereum,
            walletAddress = "0xsdlkfjskdfjlskfjslkdfjlskjf",
            type = WalletType.Multicoin,
            listPosition = ListPosition.Single,
            isCurrent = true,
            onEdit = {},
        )
    }
}
