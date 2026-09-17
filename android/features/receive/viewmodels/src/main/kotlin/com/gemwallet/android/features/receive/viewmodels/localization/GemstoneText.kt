package com.gemwallet.android.features.receive.viewmodels.localization

import android.content.Context
import com.gemwallet.android.domains.asset.networkFullName
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemReceiveWarning

internal fun GemReceiveWarning.text(context: Context, asset: Asset): String = when (this) {
    GemReceiveWarning.ASSET_NETWORK -> context.getString(
        R.string.receive_warning,
        asset.symbol.boldMarkdown(),
        asset.networkFullName.boldMarkdown(),
    )
    GemReceiveWarning.NO_DESTINATION_TAG_REQUIRED -> context.getString(R.string.wallet_receive_no_destination_tag_required)
    GemReceiveWarning.NO_MEMO_REQUIRED -> context.getString(R.string.wallet_receive_no_memo_required)
}
