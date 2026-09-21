package com.gemwallet.android.data.service.store.database.di

import android.content.Context
import androidx.room.migration.Migration

fun gemDatabaseMigrations(context: Context): Array<Migration> = arrayOf(
    Migration_71_72,
    Migration_72_73,
    Migration_73_74,
    Migration_74_75,
    Migration_75_76,
    Migration_76_77,
    Migration_77_78,
    Migration_78_79,
    Migration_79_80,
    Migration_80_81,
    Migration_81_82,
    Migration_82_83,
    Migration_83_84,
    Migration_84_85,
    Migration_85_86,
    Migration_86_87,
    Migration_87_88,
    Migration_88_89,
    Migration_89_90,
    Migration_90_91(context),
    Migration_91_92,
    Migration_92_93,
    Migration_93_94,
    Migration_94_95,
)
