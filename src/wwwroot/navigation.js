// function uploadFile(){
//     return}

function showUpload(){
    hideAllContainers();
     document.getElementById("upload-container").hidden = false;
    return;
}
function showMain(){
    hideAllContainers();
    document.getElementById("main-container").hidden = false;
    return;
}
function showAnalyseWahl(){
    hideAllContainers();
     document.getElementById("analyse-wahl-container").hidden = false;
    return;
}
function showAnalyseWahl(){
   hideAllContainers();
     document.getElementById("analyse-wahl-container").hidden = false;
    return;
}
function showResult(){
    hideAllContainers();
     document.getElementById("result-container").hidden = false;
    return;
}
function hideAllContainers(){
    document.getElementById("main-container").hidden = true;
    document.getElementById("upload-container").hidden = true;
     document.getElementById("analyse-wahl-container").hidden = true;
     document.getElementById("result-container").hidden = true;
    return;
}