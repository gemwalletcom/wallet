package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.PrimaryKey
import com.wallet.core.primitives.AssetList

@Entity(tableName = "asset_lists")
data class DbAssetList(
    @PrimaryKey val id: String,
    val name: String,
    val count: Int,
)

fun List<AssetList>.toRecord(): List<DbAssetList> = map { DbAssetList(id = it.id, name = it.name, count = it.count.toInt()) }

fun DbAssetList.toDTO(): AssetList = AssetList(id = id, name = name, count = count.toUInt())
