package com.gemwallet.android.data.services.store.database

import androidx.room.Room
import androidx.test.platform.app.InstrumentationRegistry

fun mockGemDatabase(): GemDatabase = Room.inMemoryDatabaseBuilder(
    InstrumentationRegistry.getInstrumentation().targetContext,
    GemDatabase::class.java,
).build()
