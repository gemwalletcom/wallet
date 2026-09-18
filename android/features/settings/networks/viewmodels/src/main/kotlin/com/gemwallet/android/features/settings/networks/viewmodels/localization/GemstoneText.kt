package com.gemwallet.android.features.settings.networks.viewmodels.localization

import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSubtitle
import uniffi.gemstone.GemServiceEndpointType
import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemNodeCheckRow

@StringRes
internal fun GemNodeCheckRow.stringRes(): Int = when (this) {
    is GemNodeCheckRow.ChainId -> R.string.nodes_import_node_chain_id
    is GemNodeCheckRow.InSync -> R.string.nodes_import_node_in_sync
    is GemNodeCheckRow.LatestBlock -> R.string.nodes_import_node_latest_block
    is GemNodeCheckRow.Latency -> R.string.nodes_import_node_latency
}

internal fun GemNodeCheckRow.text(context: Context): String = when (this) {
    is GemNodeCheckRow.ChainId -> value
    is GemNodeCheckRow.LatestBlock -> value
    is GemNodeCheckRow.Latency -> context.getString(R.string.common_latency_in_ms, milliseconds.toInt())
    is GemNodeCheckRow.InSync -> ""
}

internal fun GemNodeRowTitle.string(context: Context): String = text(context.getString(R.string.nodes_gem_wallet_node))

internal fun GemNodeSubtitle.text(context: Context): String = text(context.getString(R.string.nodes_import_node_latest_block))

internal fun GemServiceEndpointType.string(context: Context): String = when (this) {
    GemServiceEndpointType.API -> "API"
    GemServiceEndpointType.GEM_NODE -> context.getString(R.string.nodes_gem_wallet_node)
}

internal fun GemLatencyStatus.text(context: Context): String = when (this) {
    is GemLatencyStatus.Loading -> ""
    is GemLatencyStatus.Error -> context.getString(R.string.errors_error)
    is GemLatencyStatus.Result -> context.getString(R.string.common_latency_in_ms, latency.value.toLong())
}
