
export class StatevectorPlotter {

  constructor(canvas) {
    this._canvas = canvas;
    this._ctx = canvas.getContext("2d");
    this._paddingTop = 40;
    this._paddingRight = 50;
    this._paddingBottom = 50;
    this._paddingLeft = 50;
  }

  setPadding({ top, right, bottom, left }) {
    this._paddingTop = top ?? this._paddingTop;
    this._paddingRight = right ?? this._paddingRight;
    this._paddingBottom = bottom ?? this._paddingBottom;
    this._paddingLeft = left ?? this._paddingLeft;
  }

  plot(statevector) {
    const { qubitWidth, bases } = statevector;
    this._computeChartDimensions(qubitWidth);
    this._clearCanvas();
    this._drawAxes(qubitWidth);
    this._drawAmpitudes(statevector);
    this._drawPhases(statevector);
  }

  _computeChartDimensions(qubitWidth) {
    this._canvasWidth = this._canvas.width;
    this._canvasHeight = this._canvas.height;
    this._chartWidth = this._canvasWidth - this._paddingLeft - this._paddingRight;
    this._chartHeight = this._canvasHeight - this._paddingTop - this._paddingBottom;
    this._chartTop = this._paddingTop;
    this._chartBottom = this._canvasHeight - this._paddingBottom;
    this._chartLeft = this._paddingLeft;
    this._chartRight = this._canvasWidth - this._paddingRight;
    this._chartMiddle = this._chartHeight / 2 + this._chartTop;
    this._dx = this._chartWidth / (2 ** qubitWidth);
    this._dy = this._chartHeight / 2;
  }

  _clearCanvas() {
    this._ctx.clearRect(0, 0, this._canvasWidth, this._canvasHeight);
  }

  _drawAxes(qubitWidth) {
    const ctx = this._ctx;
    const {
      _chartTop: chartTop,
      _chartBottom: chartBottom,
      _chartLeft: chartLeft,
      _chartRight: chartRight,
      _dx: dx,
      _dy: dy,
      _chartMiddle: middleChartHeight,
    } = this;

    // Draw X-axis
    ctx.strokeStyle = "#000";
    ctx.beginPath();
    ctx.moveTo(chartLeft, middleChartHeight);
    ctx.lineTo(chartRight, middleChartHeight);
    ctx.stroke();

    // Draw Y-axes
    ctx.beginPath();
    ctx.moveTo(chartLeft, chartTop);
    ctx.lineTo(chartLeft, middleChartHeight);
    ctx.moveTo(chartRight, chartTop);
    ctx.lineTo(chartRight, chartBottom);
    ctx.stroke();

    // Draw X-axis ticks and labels
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    const halfDx = dx / 2;
    for (let i = 0; i < 2 ** qubitWidth; i++) {
      const x = i * dx + chartLeft;
      const binary = i.toString(2).padStart(qubitWidth, "0");
      ctx.beginPath();
      ctx.moveTo(x, middleChartHeight - 5);
      ctx.lineTo(x, middleChartHeight + 5);
      ctx.stroke();
      ctx.fillText(binary, x + halfDx, middleChartHeight + 10);
    }

    // Draw left Y-axis ticks and labels
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    for (let i = 0; i <= 10; i++) {
      const tickY = chartTop + (i / 10) * dy;
      const label = ((10 - i) / 10).toFixed(1);
      ctx.beginPath();
      ctx.moveTo(chartLeft - 5, tickY);
      ctx.lineTo(chartLeft, tickY);
      ctx.stroke();
      ctx.fillText(label, chartLeft - 8, tickY);
    }

    // Draw right Y-axis ticks and labels
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";

    // Draw 180° and -180° labels
    ctx.beginPath();
    ctx.moveTo(chartRight + 5, chartTop);
    ctx.lineTo(chartRight, chartTop);
    ctx.stroke();
    ctx.fillText("180°", chartRight + 8, chartTop);

    ctx.beginPath();
    ctx.moveTo(chartRight + 5, chartBottom);
    ctx.lineTo(chartRight, chartBottom);
    ctx.stroke();
    ctx.fillText("-180°", chartRight + 8, chartBottom);

    // Draw other labels
    function tickAtTheMiddle(topPos, bottomPos, topValue, bottomValue, steps, currentStep = 0) {
      if (currentStep > steps) {
        return;
      }

      const tickY = (topPos + bottomPos) / 2;
      const value = ((topValue + bottomValue) / 2);
      const label = `${value.toFixed(1)}°`;

      ctx.textAlign = "left";
      ctx.textBaseline = "middle";

      ctx.beginPath();
      ctx.moveTo(chartRight + 5, tickY);
      ctx.lineTo(chartRight, tickY);
      ctx.stroke();
      ctx.fillText(label, chartRight + 8, tickY);

      tickAtTheMiddle(topPos, tickY, topValue, value, steps, currentStep + 1);
      tickAtTheMiddle(tickY, bottomPos, value, bottomValue, steps, currentStep + 1);
    }

    tickAtTheMiddle(chartTop, chartBottom, 180.0, -180.0, 3);
  }

  _drawAmpitudes(statevector) {
    const ctx = this._ctx;
    const {
      _chartTop: chartTop,
      _chartLeft: chartLeft,
      _dx: dx,
      _dy: dy
    } = this;

    // Draw amplitude bars
    const prevFillStyle = ctx.fillStyle;
    const prevStrokeStyle = ctx.strokeStyle;
    ctx.fillStyle = "#0077be";
    ctx.strokeStyle = "#00072d";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.lineWidth = 1;
    const amplitudes = statevector.bases;
    const barWidth = dx * 0.8;
    const halfDx = dx / 2;
    const semiBar = barWidth / 2;
    for (let i = 0; i < amplitudes.length; i += 2) {
      const real = amplitudes[i];
      const imag = amplitudes[i + 1];
      const magnitude = Math.sqrt(real ** 2 + imag ** 2);
      const x = (i / 2) * dx + chartLeft + halfDx - semiBar;
      const barHeight = magnitude * dy;
      const y = chartTop + dy - barHeight;
      ctx.fillRect(x, y, barWidth, barHeight);
      ctx.strokeRect(x, y, barWidth, barHeight);

      ctx.beginPath();
      ctx.fillText(magnitude.toFixed(3), x + semiBar, y - 20);
    }
    ctx.fillStyle = prevFillStyle;
    ctx.strokeStyle = prevStrokeStyle;
  }

  _drawPhases(statevector) {
    const ctx = this._ctx;
    const {
      _chartLeft: chartLeft,
      _chartHeight: chartHeight,
      _chartMiddle: chartMiddle,
      _dx: dx
    } = this;

    // Draw phase points
    const prevFillStyle = ctx.fillStyle;
    ctx.fillStyle = "limegreen";
    const amplitudes = statevector.bases;
    const halfDx = dx / 2;
    const radius = 3;
    for (let i = 0; i < amplitudes.length; i += 2) {
      const real = amplitudes[i];
      const imag = amplitudes[i + 1];
      const phase = Math.atan2(imag, real) / Math.PI * 180; // in degrees
      const x = chartLeft + (i / 2) * dx + halfDx;
      const y = chartMiddle - phase / 360 * chartHeight;
      ctx.beginPath();
      ctx.arc(x, y, radius, 0, 2 * Math.PI);
      ctx.fill();
      ctx.fillText(phase.toFixed(2), x, y - 10);
    }
    ctx.fillStyle = prevFillStyle;
  }
}
