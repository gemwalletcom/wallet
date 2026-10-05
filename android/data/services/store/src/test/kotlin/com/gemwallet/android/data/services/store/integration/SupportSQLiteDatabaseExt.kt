package com.gemwallet.android.data.services.store.integration

import androidx.sqlite.db.SupportSQLiteDatabase
import org.junit.Assert.assertTrue

internal fun SupportSQLiteDatabase.rows(query: String): List<List<String?>> = query(query).use { cursor ->
    buildList {
        while (cursor.moveToNext()) {
            add((0 until cursor.columnCount).map { if (cursor.isNull(it)) null else cursor.getString(it) })
        }
    }
}

internal fun SupportSQLiteDatabase.longForQuery(query: String): Long {
    val cursor = query(query)
    return cursor.use {
        assertTrue(it.moveToFirst())
        it.getLong(0)
    }
}

internal fun SupportSQLiteDatabase.hasTable(name: String): Boolean {
    val cursor = query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = '$name'")
    return cursor.use { it.moveToFirst() }
}

internal fun SupportSQLiteDatabase.hasColumn(table: String, column: String): Boolean {
    val cursor = query("PRAGMA table_info($table)")
    return cursor.use {
        while (it.moveToNext()) {
            if (it.getString(1) == column) {
                return@use true
            }
        }
        false
    }
}

internal fun SupportSQLiteDatabase.hasIndex(table: String, name: String): Boolean {
    val cursor = query("PRAGMA index_list($table)")
    return cursor.use {
        while (it.moveToNext()) {
            if (it.getString(1) == name) {
                return@use true
            }
        }
        false
    }
}

internal fun SupportSQLiteDatabase.hasForeignKey(table: String, from: String, toTable: String, to: String): Boolean {
    val cursor = query("PRAGMA foreign_key_list($table)")
    return cursor.use {
        while (it.moveToNext()) {
            if (it.getString(2) == toTable && it.getString(3) == from && it.getString(4) == to) {
                return@use true
            }
        }
        false
    }
}
