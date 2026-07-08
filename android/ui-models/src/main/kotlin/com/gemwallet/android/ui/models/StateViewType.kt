package com.gemwallet.android.ui.models

sealed interface StateViewType<out T> {
    data object NoData : StateViewType<Nothing>
    data object Loading : StateViewType<Nothing>
    data class Data<T>(val data: T) : StateViewType<T>
    data object Error : StateViewType<Nothing>
}

val <T> StateViewType<T>.dataOrNull: T?
    get() = (this as? StateViewType.Data)?.data

fun <T, R> StateViewType<T>.flatMap(transform: (T) -> StateViewType<R>): StateViewType<R> = when (this) {
    StateViewType.NoData -> StateViewType.NoData
    StateViewType.Loading -> StateViewType.Loading
    StateViewType.Error -> StateViewType.Error
    is StateViewType.Data -> transform(data)
}
