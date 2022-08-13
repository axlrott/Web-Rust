var xValues = ['2022-01-01', '2022-02-01', '2022-03-01', '2022-04-01', '2022-05-01', '2022-06-01', '2022-07-01',
'2022-08-01', '2022-09-01', '2022-10-01', '2022-11-01', '2022-12-01'];
var yValues = [2, 8, 10, 12, 15, 20, 22, 25, 29, 33, 40, 42];

new Chart("lineChart", {
  type: "line",
  data: {
    labels: xValues,
    datasets: [{
      label: 'Conexiones',
      fill: false,
      backgroundColor: "rgba(0,0,255,1.0)",
      borderColor: "rgba(0,0,255,0.1)",
      data: yValues
    }]
  },
  options: {
    responsive: true,
    legend: {display: true},
    scales: {
      xAxes: [{
        type: 'time',
        distribution: 'linear',
        time: {
          displayFormats: {
            'hour': 'HH:mm:ss',
            'day': 'HH:mm',
            'week': 'MMM DD',
            'month': 'DD MMM YYYY',
            'quarter': 'MMM YYYY',
            'year': 'MMM YYYY',
         }
        }
      }]
    }
  }
});

new Chart("barChart", {
  type: "bar",
  data: {
    labels: xValues,
    datasets: [{
      label: 'Conexiones',
      fill: false,
      backgroundColor: "rgba(0,0,255,1.0)",
      borderColor: "rgba(0,0,255,0.1)",
      data: yValues
    }]
  },
  options: {
    responsive: true,
    legend: {display: true},
    scales: {
      xAxes: [{
        type: 'time',
        distribution: 'linear',
        time: {
          displayFormats: {
            'hour': 'HH:mm:ss',
            'day': 'HH:mm',
            'week': 'MMM DD',
            'month': 'MMM YYYY',
            'quarter': 'MMM YYYY',
            'year': 'MMM YYYY',
         }
        }
      }]
    }
  }
})

function alertSelectedValueLine() {
  var select = document.getElementById('longTimeLine');
  var text = select.options[select.selectedIndex].text;
  if (text == 'Last hour') {
    alert("Se va a ver la ultima hora")
  } else if (text == 'Last five hours') {
    alert("Se van a ver las ultimas 5 horas")
  } else if (text == 'Last day') {
    alert("Se va a ver el ultimo dia")
  } else {
    alert("Se va a ver los ultimos 3 dias")
    //Recarga la pagina
    location.reload()
  }
}