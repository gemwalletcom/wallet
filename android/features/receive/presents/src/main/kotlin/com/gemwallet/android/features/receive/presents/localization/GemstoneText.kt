package com.gemwallet.android.features.receive.presents.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.networkFullName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ext.boldMarkdown
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemReceiveWarning

@Composable
internal fun GemReceiveWarning.string(asset: Asset): String = when (this) {
    GemReceiveWarning.ASSET_NETWORK -> stringResource(
        R.string.receive_warning,
        asset.symbol.boldMarkdown(),
        asset.networkFullName.boldMarkdown(),
    )
    GemReceiveWarning.NO_DESTINATION_TAG_REQUIRED -> stringResource(R.string.wallet_receive_no_destination_tag_required)
    GemReceiveWarning.NO_MEMO_REQUIRED -> stringResource(R.string.wallet_receive_no_memo_required)
}
