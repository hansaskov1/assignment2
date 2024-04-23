use esp_idf_hal::{
    adc::{attenuation, AdcChannelDriver, AdcDriver, ADC1},
    gpio::Gpio34,
};

pub struct AdcTempReader<'a> {
    adc: AdcDriver<'a, ADC1>,
    adc_pin: AdcChannelDriver<'a, { attenuation::DB_6 }, Gpio34>,
}

impl<'a> AdcTempReader<'a> {
    pub fn new(
        adc: AdcDriver<'a, ADC1>,
        adc_pin: AdcChannelDriver<'a, { attenuation::DB_6 }, Gpio34>,
    ) -> anyhow::Result<Self> {
        Ok(Self { adc, adc_pin })
    }

    pub fn read_temperature(&mut self) -> anyhow::Result<f32> {
        let adc_value = self.adc.read(&mut self.adc_pin)?;
        let temperature = Self::convert_temperature(adc_value);

        Ok(temperature)
    }

    // See datasheet https://www.ti.com/lit/ds/symlink/lmt86.pdf for the conversion at page 10
    // See wolframalpha for a simplified version https://www.wolframalpha.com/input?i=%2810.888+-+sqrt%28%2810.888+*+10.888%29+%2B+4.+*+0.00347+*+%281777.3+-+x+%29+%29%29+%2F%28+2.+*+%28-0.00347%29%29+%2B+30
    pub fn convert_temperature(adc_value: u16) -> f32 {
        -1538.88 + 144.092 * f32::sqrt(143.217 - 0.01388 * adc_value as f32)
    }
}
