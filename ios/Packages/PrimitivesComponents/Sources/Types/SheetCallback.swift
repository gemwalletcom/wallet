// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol SheetRejectable: Sendable {
    var id: String { get }
    func reject(_ error: any Error)
}

public final class SheetCallback<T: Identifiable & Sendable>: SheetRejectable, Identifiable where T.ID == String {
    public let payload: T
    public let delegate: StringResultAction

    public init(
        payload: T,
        delegate: @escaping StringResultAction,
    ) {
        self.payload = payload
        self.delegate = delegate
    }

    public var id: String {
        payload.id
    }

    public func reject(_ error: any Error) {
        delegate(.failure(error))
    }
}
