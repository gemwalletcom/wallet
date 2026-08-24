package com.gemwallet.android.ui.components.list_item.property

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.networkFullName
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain

@Composable
fun PropertyNetworkItem(
    chain: Chain,
    value: String = chain.networkName(),
    listPosition: ListPosition = ListPosition.Single,
    onOpenNetwork: (() -> Unit)? = null
) {
    val asset = chain.asset()
    PropertyItem(
        modifier = onOpenNetwork?.let {
            Modifier.clickable(onClick = it)
        } ?: Modifier,
        title = { PropertyTitleText(R.string.transfer_network) },
        data = {
            PropertyDataText(
                text = value,
                badge = { DataBadgeChevron(asset.chain.getIconUrl(), onOpenNetwork != null) }
            )
        },
        listPosition = listPosition,
    )
}

@Composable
fun PropertyNetworkItem(
    asset: Asset,
    listPosition: ListPosition = ListPosition.Single,
    onOpenNetwork: (() -> Unit)? = null
) {
    PropertyNetworkItem(
        chain = asset.chain,
        value = asset.networkFullName,
        listPosition = listPosition,
        onOpenNetwork = onOpenNetwork
    )
}
