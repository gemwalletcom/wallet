// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import SwiftUI

public extension View {
    func alert<T>(
        _ title: some StringProtocol,
        presenting data: Binding<T?>,
        sensoryFeedback: SensoryFeedback? = nil,
        @ViewBuilder actions: (T) -> some View,
        @ViewBuilder message: () -> some View = { EmptyView() },
    )
        -> some View
    {
        let isPresented: Binding<Bool> = Binding(
            get: { data.wrappedValue != nil },
            set: { newValue in
                guard !newValue else { return }
                data.wrappedValue = nil
            },
        )

        return alert(
            title,
            isPresented: isPresented,
            presenting: data.wrappedValue,
            actions: { value in
                VStack {
                    actions(value)
                    Button(Localized.Common.cancel, role: .cancel) {
                        isPresented.wrappedValue = false
                    }
                }
            },
            message: { _ in
                message()
            },
        )
        .ifLet(sensoryFeedback) { view, value in
            view.sensoryFeedback(value, trigger: isPresented.wrappedValue) { $1 }
        }
    }
}
