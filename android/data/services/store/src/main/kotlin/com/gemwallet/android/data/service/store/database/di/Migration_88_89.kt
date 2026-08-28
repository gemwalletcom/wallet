package com.gemwallet.android.data.service.store.database.di

import android.content.Context
import androidx.core.content.edit
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState

class Migration_88_89(private val context: Context) : Migration(88, 89) {
    override fun migrate(db: SupportSQLiteDatabase) {
        val nodes = mutableMapOf<String, MutableList<Node>>()
        db.query("SELECT url, status, priority, chain FROM nodes").use { cursor ->
            while (cursor.moveToNext()) {
                nodes.getOrPut(cursor.getString(3)) { mutableListOf() }
                    .add(Node(cursor.getString(0), NodeState.valueOf(cursor.getString(1)), cursor.getInt(2)))
            }
        }
        context.getSharedPreferences("node-config", Context.MODE_PRIVATE).edit(commit = true) {
            nodes.forEach { (chain, chainNodes) -> putString("nodes-$chain", chainNodes.toJson()) }
        }
        db.execSQL("DROP TABLE IF EXISTS `nodes`")
    }
}
