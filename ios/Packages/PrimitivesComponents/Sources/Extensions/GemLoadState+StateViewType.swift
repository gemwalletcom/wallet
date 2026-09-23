// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemLoadState

public extension GemLoadState {
    private func stateViewType<T>(_ value: T?) -> StateViewType<T> {
        switch self {
        case .noData: .noData
        case .loading: value.map { .data($0) } ?? .loading
        case .data: value.map { .data($0) } ?? .noData
        case let .error(error): .error(error)
        }
    }

    func stateViewType<T>(_ values: [T]) -> StateViewType<[T]> {
        stateViewType(values.isEmpty ? nil : values)
    }
}
