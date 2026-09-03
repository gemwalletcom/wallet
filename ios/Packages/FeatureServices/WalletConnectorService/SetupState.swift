// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor SetupState {
    private var task: Task<Void, Never>?

    deinit {
        task?.cancel()
    }

    func start(_ makeTask: @Sendable () -> Task<Void, Never>) {
        guard task == nil else { return }
        task = makeTask()
    }
}
