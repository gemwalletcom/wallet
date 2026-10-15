package com.gemwallet.android.data.services.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase

internal fun gemDatabaseMigrationTestHelper(): MigrationTestHelper = MigrationTestHelper(
    InstrumentationRegistry.getInstrumentation(),
    GemDatabase::class.java,
    emptyList(),
    FrameworkSQLiteOpenHelperFactory(),
)
