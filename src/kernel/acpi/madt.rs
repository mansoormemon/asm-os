// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use acpi::AcpiError;
use acpi::InterruptModel;
use acpi::madt::Madt;
use acpi::platform::ProcessorInfo;
use conquer_once::spin::OnceCell;

///////////////////
// Cached Values
///////////////////

static INTERRUPT_MODEL: OnceCell<Option<InterruptModel>> = OnceCell::uninit();
static PROCESSOR_INFO: OnceCell<Option<ProcessorInfo>> = OnceCell::uninit();

pub(super) fn read(sdt: &Madt) -> Result<(), AcpiError> {
    let (interrupt_model, processor_info) = sdt.parse_interrupt_model()?;

    INTERRUPT_MODEL.try_init_once(
        || { Some(interrupt_model) }
    ).expect("failed to initialize interrupt model");

    PROCESSOR_INFO.try_init_once(
        || { processor_info }
    ).expect("failed to initialize processor info");

    Ok(())
}

pub fn get_interrupt_model() -> Option<&'static InterruptModel> { INTERRUPT_MODEL.try_get().unwrap_or(&None).as_ref() }

pub fn get_processor_info() -> Option<&'static ProcessorInfo> { PROCESSOR_INFO.try_get().unwrap_or(&None).as_ref() }
