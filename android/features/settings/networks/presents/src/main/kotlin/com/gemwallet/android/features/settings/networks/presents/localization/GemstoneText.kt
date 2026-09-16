package com.gemwallet.android.features.settings.networks.presents.localization

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemChainSettingsSection
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSubtitle
import uniffi.gemstone.GemServiceEndpointType

@Composable
internal fun GemNodeRowTitle.string(): String = when (this) {
    is GemNodeRowTitle.Host -> host
    is GemNodeRowTitle.GemNode -> "${stringResource(R.string.nodes_gem_wallet_node)} $flag"
}

@Composable
internal fun GemServiceEndpointType.string(): String = when (this) {
    GemServiceEndpointType.API -> "API"
    GemServiceEndpointType.GEM_NODE -> stringResource(R.string.nodes_gem_wallet_node)
}

@StringRes
internal fun GemChainSettingsSection.stringRes(): Int = when (this) {
    GemChainSettingsSection.NODES -> R.string.settings_networks_source
    GemChainSettingsSection.EXPLORER -> R.string.settings_networks_explorer
}

@StringRes
internal fun GemNodeCheckRow.stringRes(): Int = when (this) {
    is GemNodeCheckRow.ChainId -> R.string.nodes_import_node_chain_id
    is GemNodeCheckRow.InSync -> R.string.nodes_import_node_in_sync
    is GemNodeCheckRow.LatestBlock -> R.string.nodes_import_node_latest_block
    is GemNodeCheckRow.Latency -> R.string.nodes_import_node_latency
}

@Composable
internal fun GemNodeCheckRow.text(): String = when (this) {
    is GemNodeCheckRow.ChainId -> value
    is GemNodeCheckRow.LatestBlock -> value
    is GemNodeCheckRow.Latency -> stringResource(R.string.common_latency_in_ms, milliseconds.toInt())
    is GemNodeCheckRow.InSync -> ""
}

@StringRes
internal fun GemNodeSubtitle.stringRes(): Int = when (this) {
    is GemNodeSubtitle.LatestBlock -> R.string.nodes_import_node_latest_block
}
