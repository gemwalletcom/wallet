// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public extension View {
    func copyToast(
        model: CopyTypeViewModel,
        isPresenting: Binding<Bool>,
        feedbackGenerator: UINotificationFeedbackGenerator = UINotificationFeedbackGenerator(),
    ) -> some View {
        toast(isPresenting: isPresenting, message: ToastMessage(title: model.message, image: model.systemImage))
            .onChange(of: isPresenting.wrappedValue, initial: true) { _, newValue in
                if newValue {
                    model.copy()
                    feedbackGenerator.notificationOccurred(.success)
                }
            }
    }
}
