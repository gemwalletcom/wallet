package com.gemwallet.android.ext

import kotlinx.coroutines.CancellationException

suspend fun <T> runCatchingCancellable(block: suspend () -> T): Result<T> = try {
    Result.success(block())
} catch (error: CancellationException) {
    throw error
} catch (error: Throwable) {
    Result.failure(error)
}
