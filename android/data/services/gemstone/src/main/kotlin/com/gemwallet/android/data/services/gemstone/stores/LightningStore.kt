package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.LightningDao
import com.gemwallet.android.data.services.store.database.entities.DbLightningLeaf
import com.gemwallet.android.data.services.store.database.entities.DbLightningPendingSend
import uniffi.gemstone.GemLightningStore
import uniffi.gemstone.LightningLeaf
import uniffi.gemstone.LightningPendingSend

class GemstoneLightningStore(private val lightningDao: LightningDao) : GemLightningStore {

    override suspend fun getLeaves(walletId: String): List<LightningLeaf> = lightningDao.getLeaves(walletId).map { LightningLeaf(id = it.id, data = it.data) }

    override suspend fun updateLeaves(walletId: String, leaves: List<LightningLeaf>, deleteIds: List<String>) {
        lightningDao.updateLeaves(walletId, leaves.map { DbLightningLeaf(walletId = walletId, id = it.id, data = it.data) }, deleteIds)
    }

    override suspend fun getPendingSends(walletId: String): List<LightningPendingSend> = lightningDao.getPendingSends(walletId).map {
        LightningPendingSend(transferId = it.transferId, invoice = it.invoice, amountSats = it.amountSats?.toULong(), expiresAt = it.expiresAt.toULong())
    }

    override suspend fun addPendingSend(walletId: String, send: LightningPendingSend) {
        lightningDao.addPendingSend(
            DbLightningPendingSend(
                walletId = walletId,
                transferId = send.transferId,
                invoice = send.invoice,
                amountSats = send.amountSats?.toLong(),
                expiresAt = send.expiresAt.toLong(),
            ),
        )
    }

    override suspend fun deletePendingSend(walletId: String, transferId: String) {
        lightningDao.deletePendingSend(walletId, transferId)
    }
}
