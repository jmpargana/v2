/* g8 teaching workspace — bytecode stepper widget */

function initSteppers() {
  document.querySelectorAll('.stepper').forEach(stepper => {
    const steps = JSON.parse(stepper.dataset.steps);
    const instrLines = stepper.querySelectorAll('.stepper-instructions div');
    const stackContainer = stepper.querySelector('.stepper-stack');
    const status = stepper.querySelector('.stepper-status');
    const prevBtn = stepper.querySelector('[data-prev]');
    const nextBtn = stepper.querySelector('[data-next]');
    let current = -1;

    function render() {
      instrLines.forEach((line, i) => {
        line.classList.remove('active', 'done');
        if (i === current) line.classList.add('active');
        else if (i < current) line.classList.add('done');
      });

      const cells = stackContainer.querySelectorAll('.stack-cell');
      cells.forEach(c => c.remove());

      if (current >= 0 && current < steps.length) {
        const stack = steps[current].stack;
        stack.forEach((val, i) => {
          const cell = document.createElement('div');
          cell.className = 'stack-cell' + (i === stack.length - 1 ? ' highlight' : '');
          cell.textContent = val;
          stackContainer.appendChild(cell);
        });
      }

      status.textContent = current < 0
        ? 'ready'
        : current < steps.length
          ? steps[current].label
          : 'done';

      prevBtn.disabled = current < 0;
      nextBtn.disabled = current >= steps.length - 1;
    }

    prevBtn.addEventListener('click', () => { if (current > -1) { current--; render(); } });
    nextBtn.addEventListener('click', () => { if (current < steps.length - 1) { current++; render(); } });

    render();
  });
}

document.addEventListener('DOMContentLoaded', initSteppers);
