/* g8 teaching workspace — quiz widget */

function initQuizzes() {
  document.querySelectorAll('.quiz').forEach(quiz => {
    const correctIndex = parseInt(quiz.dataset.correct, 10);
    const feedback = quiz.querySelector('.quiz-feedback');
    const options = quiz.querySelectorAll('.quiz-option');

    options.forEach((opt, i) => {
      opt.addEventListener('click', () => {
        if (quiz.classList.contains('answered')) return;
        quiz.classList.add('answered');

        options.forEach(o => o.classList.add('disabled'));

        if (i === correctIndex) {
          opt.classList.add('correct');
          feedback.textContent = quiz.dataset.right || 'Correct.';
        } else {
          opt.classList.add('wrong');
          options[correctIndex].classList.add('correct');
          feedback.textContent = quiz.dataset.wrong || 'Not quite — see the highlighted answer.';
        }
        feedback.classList.add('visible');
      });
    });
  });
}

document.addEventListener('DOMContentLoaded', initQuizzes);
