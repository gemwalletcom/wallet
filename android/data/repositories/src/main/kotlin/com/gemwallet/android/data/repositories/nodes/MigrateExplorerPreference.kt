package com.gemwallet.android.data.repositories.nodes

import android.content.Context
import androidx.core.content.edit
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemExplorerService

class MigrateExplorerPreference(
    private val context: Context,
    private val explorerService: GemExplorerService,
) {
    suspend operator fun invoke() = withContext(Dispatchers.IO) {
        val legacyStore = context.getSharedPreferences("node-config", Context.MODE_PRIVATE)
        Chain.entries.forEach { chain ->
            val key = "current_explorer-${chain.string}"
            val name = legacyStore.getString(key, null)?.takeIf { it.isNotEmpty() } ?: return@forEach
            explorerService.setExplorerName(chain.string, name)
            legacyStore.edit(commit = true) { remove(key) }
        }
    }
}
