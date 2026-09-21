package com.gemwallet.android.features.settings.networks.viewmodels.localization

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSubtitle

@StringRes
internal fun GemNodeCheckRow.stringRes(): Int = when (this) {
    is GemNodeCheckRow.ChainId -> R.string.nodes_import_node_chain_id
    is GemNodeCheckRow.InSync -> R.string.nodes_import_node_in_sync
    is GemNodeCheckRow.LatestBlock -> R.string.nodes_import_node_latest_block
    is GemNodeCheckRow.Latency -> R.string.nodes_import_node_latency
}

internal fun GemNodeCheckRow.text(context: Context): String = when (this) {
    is GemNodeCheckRow.ChainId -> value
    is GemNodeCheckRow.LatestBlock -> value.text()
    is GemNodeCheckRow.Latency -> context.getString(R.string.common_latency_in_ms, milliseconds.toInt())
    is GemNodeCheckRow.InSync -> ""
}

internal fun GemNodeRowTitle.string(context: Context): String = text(context.getString(R.string.nodes_gem_wallet_node))

internal fun GemNodeSubtitle.text(context: Context): String = when (this) {
    is GemNodeSubtitle.LatestBlock -> text(context.getString(R.string.nodes_import_node_latest_block), value?.text())
}
